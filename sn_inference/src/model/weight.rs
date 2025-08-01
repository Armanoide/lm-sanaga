use crate::config::config::Config;
use crate::error::{Error, Result};
use crate::utils::d_type::DTypeExt;
use crate::utils::string::{find_json_object_end_bytes};
use glob::glob;
use memmap2::MmapOptions;
use mlx_rs::{Array, Dtype};
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::c_void;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::vec::Vec;
use tracing::{debug, error};

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
    pub data: Array,
}
#[derive(Debug)]
pub struct Metadata {
    pub format: Option<String>,
}
#[derive(Debug)]
pub struct Weight {
    pub metadata: Metadata,
    pub tensors: HashMap<String, Tensor>,
}

impl Weight {
    pub fn new(config: &Config) -> Result<Self> {
        let weights_files = Weight::find_model_files(&config.root_path)?;
        Weight::load_weights(&weights_files)
    }

    fn read_safetensors_header(
        mmap: &memmap2::Mmap,
        buffer: &mut Vec<u8>,
    ) -> Result<(WeightJSON, usize)> {

        if mmap.len() < HEADER_MAX_SAFETENSORS {
            return Err(Error::SafetensorsHeaderReadError);
        }
        // Read len_bytes if needed (depending on your format)
        let buffer = &mmap[HEADER_OFFSET_SAFETENSORS..HEADER_MAX_SAFETENSORS];

        let real_size_header = find_json_object_end_bytes(buffer);
        match real_size_header {
            Some(size_header) => {
                let buffer_slice = &buffer[..size_header];
                let json_str = String::from_utf8_lossy(&buffer_slice)
                    .trim_end()
                    .to_string();
                let json: WeightJSON = serde_json::from_str(&json_str)?;
                Ok((json, size_header))
            }
            None => Err(Error::SafetensorsHeaderReadError),
        }
    }

    fn read_safetensors_weights(
        mmap: &memmap2::Mmap,
        weights_json: WeightJSON,
        offset_header: usize,
    ) -> Result<HashMap<String, Tensor>> {
        let mut weights: HashMap<String, Tensor> = HashMap::new();


        let base_offset = offset_header + HEADER_OFFSET_SAFETENSORS;

        for (name, weight) in weights_json.tensors {
            let dtype = Dtype::from_string_unsafe(&weight.dtype);
            let shape = weight.shape.clone();
            let data_offsets = weight.data_offsets;
            let offset_start = base_offset + data_offsets[0] as usize;
            let offset_end = base_offset + data_offsets[1] as usize;

            if offset_end > mmap.len() {
                return Err(Error::SafetensorsOutOfBounds {
                    tensor: name.clone(),
                    start: offset_start,
                    end: offset_end,
                    file_size: mmap.len(),
                });
            }

            let data_slice = &mmap[offset_start..offset_end];

            debug!(
                "Loading tensor '{}': dtype={:?} shape={:?} size={} bytes",
                name,
                dtype,
                shape,
                offset_end - offset_start
            );

            let data = unsafe {
                Array::from_raw_data(data_slice.as_ptr() as *const c_void, &shape, dtype)
            };

            weights.insert(
                name,
                Tensor {
                    data,
                    shape,
                    dtype,
                    size: (offset_end - offset_start) as u64,
                },
            );
        }

        Ok(weights)
    }

    fn load_weights(files: &Vec<String>) -> Result<Weight> {
        let mut list: Vec<Weight> = Vec::new();
        // Preallocate buffer once, reuse it for each file
        let mut buffer_header = vec![0u8; HEADER_MAX_SAFETENSORS];
        let mut total_expected_tensors: usize = 0;

        for file_path in files {

            let file = File::open(file_path).map_err(|_| Error::FileOpenError(file_path.to_owned()))?;

            // Memory-map the entire file
            let mmap = unsafe { MmapOptions::new().map(&file)? };

            // Read header into buffer
            let (weight_json, header_size) =
                Self::read_safetensors_header(&mmap, &mut buffer_header)?;

            total_expected_tensors += weight_json.tensors.len();

            let metadata_format = weight_json.metadata.as_ref().and_then(|m| m.format.clone());

            let tensors = match Self::read_safetensors_weights(&mmap, weight_json, header_size)
            {
                Ok(t) => Ok(t),
                Err(e) => {
                    error!("Failed to read weights from {}: {}", file_path, e);
                    Err(e)
                }
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
            return Err(Error::TensorSizeMismatch(
                total_expected_tensors,
                result_weights.tensors.len(),
            ));
        }
        Ok(result_weights)
    }

    fn merge_weights(list: Vec<Weight>) -> Weight {
        let mut merged_tensors = HashMap::new();
        let mut merged_metadata = Metadata { format: None };

        for weight in list {
            // Take the first non-None metadata
            if merged_metadata.format.is_none() {
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
            .map(|p| p.display().to_string())
            .collect();

        if result.is_empty() {
            return Err(Error::ModelWeightPathNotFound(weights_path));
        }
        Ok(result)
    }

    fn find_model_files(model_path: &str) -> Result<Vec<String>> {
        match Weight::find_model(model_path, &"model*.safetensors") {
            Ok(files) => Ok(files),
            Err(_) => {
                // Sometimes model files can be in weight
                Weight::find_model(model_path, &"weight*.safetensors")
            }
        }
    }
}
