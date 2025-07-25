use mlx_rs::ops::{arange, power, r#where, gt, lt, logical_and};
use mlx_rs::{rope, Array};
use crate::config::config_models::llama::{LLaMARopeScalingConfig};
use crate::error::{Error, Result};

const PI: f32 = std::f64::consts::PI as f32;
#[derive(Clone, Debug)]
pub struct RopeLlama {
    fregs: Array,
    dims: i32,
    traditional: bool,
    max_position_embeddings: i32
}

impl RopeLlama {
    pub fn new(dims: i32, base: f32, traditional: bool, rope_config: &LLaMARopeScalingConfig) -> Result<RopeLlama> {

        let dims = dims;
        let max_position_embeddings = rope_config.original_max_position_embeddings;
        let traditional = traditional;

        let factor = rope_config.factor;
        let low_freq_factor = rope_config.low_freq_factor;
        let high_freq_factor = rope_config.high_freq_factor;
        let old_context_len = rope_config.original_max_position_embeddings;

        let low_freq_wavelen = old_context_len as f32/ low_freq_factor;
        let high_freq_wavelen = old_context_len as f32 / high_freq_factor;

        let base_freqs = {
        let indices = arange::<_, f32>(0.0, dims as f32, 2.0)?;
        let exponent = &indices / (dims as f32);
            let base_arr = Array::from_f32(base);
            power(&base_arr, &exponent)?
        };

        // Step 2: Compute wavelengths
        let two_pi = Array::from_f32(2.0 * PI);
        let wavelens = &two_pi * &base_freqs;

        // Step 3: Conditionally scale freqs
        let freqs = {
            let mask = gt(&wavelens, &Array::from_f32(low_freq_wavelen))?;
            let scaled = &base_freqs * factor;
            r#where(&mask, &scaled, &base_freqs)?
        };

        // Step 4: Compute medium frequency mask
        let is_medium_freq = {
            let above_low = gt(&wavelens, &Array::from_f32(low_freq_wavelen))?;
            let below_high = lt(&wavelens, &Array::from_f32(high_freq_wavelen))?;
            logical_and(&above_low, &below_high)?
        };

        let smooth_factors = (Array::from_f32(old_context_len as f32) / wavelens - low_freq_factor) / (
            high_freq_factor - low_freq_factor
        );

        let smooth_freqs = &freqs / ((Array::from_f32(1.0) - &smooth_factors) / factor + &smooth_factors);


        let fregs_final = r#where(is_medium_freq, smooth_freqs, freqs)?;
        Ok(RopeLlama {
            fregs: fregs_final,
            dims,
            max_position_embeddings,
            traditional,
        })

    }

    pub fn forward(&self, x: &Array, offset: i32) -> Result<Array> {
        rope!(array = x, dimensions = self.dims, traditional = self.traditional, scale = 1.0, offset = offset, freqs = &self.fregs)
            .map_err(|e| Error::ExceptionMLX(e))
        
    }

}
