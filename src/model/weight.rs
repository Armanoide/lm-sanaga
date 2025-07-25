use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use glob::glob;
use crate::error::{Error, Result};
use std::vec::Vec;
use serde::Deserialize;
use mlx_rs::{Array, Dtype};
use crate::utils::string::find_json_object_end;
use crate::utils::d_type::DTypeExt;
use std::ffi::c_void;
use log::{debug, error};
use crate::config::config::Config;

static HEADER_MAX_SAFETENSORS: usize = 100_000_000;
static HEADER_OFFSET_SAFETENSORS: usize = 8;

#[derive(Debug, Deserialize)]
pub struct TensorJSON {
    pub data_offsets: [u64; 2],
    pub dtype: String,
    pub shape: Vec<i32>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct MetadataJSON {
    pub format: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct WeightJSON {
    #[serde(rename = "__metadata__")]
    pub metadata: Option<MetadataJSON>,

    #[serde(flatten)]
    pub tensors: HashMap<String, TensorJSON>,
}
#[allow(unused_variables)]
#[derive(Debug)]
pub struct Tensor {
    pub size: u64,
    pub dtype: Dtype,
    pub shape: Vec<i32>,
    pub data: Array
}
pub struct Metadata {
    pub format: Option<String>,
}
pub struct Weight {
    pub metadata: Metadata,
    pub tensors: HashMap<String, Tensor>,
}


impl Weight {
    pub fn new(config: &Config) -> Result<Self> {
        let weights_files = Weight::find_model_files(&config.root_path)?;
        Weight::load_weights(&weights_files)
    }

    fn read_safetensors_header(file_path: &str, buffer: &mut Vec<u8>) -> Result<(WeightJSON, usize)> {
        let mut file = File::open(file_path)?;
        let mut len_bytes = [0u8; HEADER_OFFSET_SAFETENSORS];

        file.read_exact(&mut len_bytes)?;
        file.read_exact(buffer)?;

        let header_str_with_noise  = String::from_utf8_lossy(&buffer).trim_end().to_string();
        let real_size_header = find_json_object_end(&header_str_with_noise);

        match real_size_header {
            Some(size_header) => {
                let buffer_slice = &buffer[..size_header];
                let json_str = String::from_utf8_lossy(&buffer_slice).trim_end().to_string();
                let json: WeightJSON = serde_json::from_str(&json_str)?;
                Ok((json, size_header))
            },
            None => {
                Err(Error::SafetensorsHeaderReadError)
            }
        }
    }


    fn read_safetensors_weights(file_path: &str, weights_json: WeightJSON, offset_header: usize) -> Result<HashMap<String, Tensor>> {
        let mut weights: HashMap<String, Tensor> = HashMap::new();

        let mut file = File::open(file_path)
            .map_err(|_| Error::FileOpenError(file_path.to_owned()))?;

        for (name, weight) in weights_json.tensors {

            let dtype = Dtype::from_string_unsafe(&weight.dtype);
            let shape = weight.shape.clone();
            let data_offsets = weight.data_offsets.clone();
            let base_offset = offset_header + HEADER_OFFSET_SAFETENSORS;
            let offset_start = base_offset + data_offsets[0] as usize;
            let offset_end = base_offset + data_offsets[1] as usize;
            let size = offset_end - offset_start;
            let mut buffer = vec![0u8; size];

            file.rewind()?;
            file.seek(SeekFrom::Start(offset_start as u64))?;
            file.read_exact(&mut buffer)?;

            debug!("Loading tensor '{}': dtype={:?} dtype={:?} shape={:?} size={} bytes",
                name, dtype, weight.dtype, shape, size);

            let data = unsafe {
                Array::from_raw_data(
                    buffer.as_ptr() as *const c_void,
                    &shape,
                    dtype,
                )
            };

            weights.insert(name, Tensor { data, shape, dtype, size: size as u64 });
        }
        Ok(weights)
    }

    fn load_weights(files: &Vec<String>) -> Result<Weight> {
        let mut list: Vec<Weight> = Vec::new();
        // Preallocate buffer once, reuse it for each file
        let mut buffer_header = vec![0u8; HEADER_MAX_SAFETENSORS];
        let mut total_expected_tensors: usize = 0;

        for file_path in files {
            // Read header into buffer
            let (weight_json, header_size) = Self::read_safetensors_header(file_path, &mut buffer_header)?;

            total_expected_tensors += weight_json.tensors.len();

            let metadata_format = weight_json.metadata.as_ref().and_then(|m| m.format.clone());

            let tensors =  match Self::read_safetensors_weights(file_path, weight_json, header_size) {
                Ok(t) => Ok(t),
                Err(e) => {
                    error!("Failed to read weights from {}: {}", file_path, e);
                    Err(e)
                },
            }?;

            list.push(Weight {
                tensors,
                metadata: Metadata {
                    format: metadata_format,
                },
            });
        }

        let result_weights = Weight::merge_weights(list);

        if result_weights.tensors.is_empty() {
            return Err(Error::NoTensorInModelFile);
        } else if total_expected_tensors != result_weights.tensors.len() {
            return Err(Error::TensorSizeMismatch(total_expected_tensors, result_weights.tensors.len()));
        }
        Ok(result_weights)
    }

    fn merge_weights(list: Vec<Weight>) -> Weight {
        let mut merged_tensors = HashMap::new();
        let mut merged_metadata = Metadata { format: None } ;

        for weight in list {
            // Take the first non-None metadata
            if  merged_metadata.format.is_none() {
                merged_metadata = weight.metadata;
            }

            // Extend the tensor map
            merged_tensors.extend(weight.tensors);
        }

        Weight {
            metadata: merged_metadata,
            tensors: merged_tensors,
        }
    }

    fn find_model(model_path: &str, regex_files: &str) -> Result<Vec<String>> {
        let pattern = Path::new(model_path).join(regex_files);
        let weights_path = pattern.display().to_string();

        let paths_results = glob(&weights_path)?;

        let result: Vec<String> = paths_results
            .filter_map(|i| i.ok())
            .map(|p| {p.display().to_string()})
            .collect();

        if result.is_empty() {
            return Err(Error::ModelWeightPathNotFound(weights_path));
        }
        Ok(result)
    }

    fn find_model_files(model_path: &str) -> Result<Vec<String>> {
        match Weight::find_model(model_path,&"model*.safetensors")  {
            Ok(files) => Ok(files),
            Err(_) => {
                // Sometimes model files can be in weight
                Weight::find_model(model_path,&"weight*.safetensors")
            },
        }
    }
}
