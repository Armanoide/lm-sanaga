use std::sync::Arc;
use crate::config::config_models::llama::LLaMARopeScalingConfig;
use crate::error::{Error, Result};
use mlx_rs::ops::{arange, arange_device, gt, gt_device, logical_and, logical_and_device, lt, lt_device, power, power_device, r#where, where_device};
use mlx_rs::{Array, rope, Stream};

const PI: f32 = std::f64::consts::PI as f32;
#[derive(Clone, Debug)]
pub struct RopeLlama {
    fregs: Array,
    dims: i32,
    traditional: bool,
    max_position_embeddings: i32,
}

impl RopeLlama {
    pub fn new(
        dims: i32,
        base: f32,
        traditional: bool,
        rope_config: &LLaMARopeScalingConfig,
        stream: Option<Arc<Stream>>
    ) -> Result<RopeLlama> {
        let dims = dims;
        let max_position_embeddings = rope_config.original_max_position_embeddings;
        let traditional = traditional;

        let factor = rope_config.factor;
        let low_freq_factor = rope_config.low_freq_factor;
        let high_freq_factor = rope_config.high_freq_factor;
        let old_context_len = rope_config.original_max_position_embeddings;

        let low_freq_wavelen = old_context_len as f32 / low_freq_factor;
        let high_freq_wavelen = old_context_len as f32 / high_freq_factor;

        let base_freqs = {
            let indices = if let Some(stream) = stream.clone() {
                arange_device::<_, f32>(0.0, dims as f32, 2.0, stream)?
            } else {
                arange::<_, f32>(0.0, dims as f32, 2.0)?
            };
            let exponent = &indices / (dims as f32);
            let base_arr = Array::from_f32(base);
            if let Some(stream) = stream.clone() {
                power_device(&base_arr, &exponent, stream)?
            } else {
                power(&base_arr, &exponent)?
            }
        };

        // Step 2: Compute wavelengths
        let two_pi = Array::from_f32(2.0 * PI);
        let wavelens = &two_pi * &base_freqs;

        // Step 3: Conditionally scale freqs
        let freqs = {
            let mask = if let Some(stream) = stream.clone() {
                gt_device(&wavelens, &Array::from_f32(low_freq_wavelen), stream)?
            } else {
                gt(&wavelens, &Array::from_f32(low_freq_wavelen))?
            };
            let scaled = &base_freqs * factor;
            if let Some(stream) = stream.clone() {
                r#where_device(&mask, &scaled, &base_freqs, stream)?

            } else {
                r#where(&mask, &scaled, &base_freqs)?
            }
        };

        // Step 4: Compute medium frequency mask
        let is_medium_freq = {
            let above_low = if let Some(stream) = stream.clone() {
                gt_device(&wavelens, &Array::from_f32(low_freq_wavelen), stream)?
            } else {
                gt(&wavelens, &Array::from_f32(low_freq_wavelen))?
            };
            let below_high = if let Some(stream) = stream.clone() {
                lt_device(&wavelens, &Array::from_f32(high_freq_wavelen), stream)?
            } else {
                 lt(&wavelens, &Array::from_f32(high_freq_wavelen))?
            };
            if let Some(stream) = stream.clone() {
                logical_and_device(&above_low, &below_high, stream)?
            } else {
                logical_and(&above_low, &below_high)?
            }
        };

        let smooth_factors = (Array::from_f32(old_context_len as f32) / wavelens - low_freq_factor)
            / (high_freq_factor - low_freq_factor);

        let smooth_freqs =
            &freqs / ((Array::from_f32(1.0) - &smooth_factors) / factor + &smooth_factors);


        let fregs_final = if let Some(stream) = stream {
            r#where_device(is_medium_freq, smooth_freqs, freqs, stream)?
        } else {
            r#where(is_medium_freq, smooth_freqs, freqs)?
        };

        Ok(RopeLlama {
            fregs: fregs_final,
            dims,
            max_position_embeddings,
            traditional,
        })
    }

    pub fn forward(&self, x: &Array, offset: i32, stream: Option<Arc<Stream>>) -> Result<Array> {
        if let Some(stream) = stream.clone() {
            rope!(
                array = x,
                dimensions = self.dims,
                traditional = self.traditional,
                scale = 1.0,
                offset = offset,
                freqs = &self.fregs,
                stream = stream
            ).map_err(|e| Error::ExceptionMLX(e))
        } else {
            rope!(
                array = x,
                dimensions = self.dims,
                traditional = self.traditional,
                scale = 1.0,
                offset = offset,
                freqs = &self.fregs
            ).map_err(|e| Error::ExceptionMLX(e))

        }
    }
}
