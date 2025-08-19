use crate::cache::k_v_cache::k_v_cache::ArcCacheItem;
use crate::config::config_models::qwen3::Qwen3Config;
use crate::error::{Error, Result};
use crate::mask::mask::AttentionMask;
use crate::model::models::qwen3::rope::RopeQwen3;
use crate::model::weight::Tensor;
use crate::module::Module;
use crate::quantized::Quantize;
use crate::safe_quantize;
use crate::utils::maybe_quantized::MaybeQuantizedLinear;
use crate::utils::maybe_quantized::QuantizableParam;
use crate::utils::rms_norm::NormExt;
use crate::utils::scaled_dot_product_attention::scaled_dot_product_attention;
use mlx_rs::Array;
use mlx_rs::builder::Builder;
use mlx_rs::module::Module as MLXModule;
use mlx_rs::nn::{Linear, LinearBuilder, RmsNorm, RmsNormBuilder};
use mlx_rs::quantization::MaybeQuantized;
use sn_core::utils::rw_lock::RwLockExt;
use std::rc::Rc;
#[derive(Clone, Debug)]
pub struct AttentionQwen3 {
    n_heads: i32,
    n_kv_heads: i32,
    scale: f64,

    q_proj: MaybeQuantized<Linear>,
    k_proj: MaybeQuantized<Linear>,
    v_proj: MaybeQuantized<Linear>,
    o_proj: MaybeQuantized<Linear>,
    rope: RopeQwen3,
    q_norm: RmsNorm,
    k_norm: RmsNorm,
}

impl Quantize for AttentionQwen3 {
    fn quantize(&mut self, group_size: i32, bits: i32) -> Result<()> {
        safe_quantize!(self, group_size, bits, q_proj, k_proj, v_proj, o_proj,);
        Ok(())
    }
}

impl Module for AttentionQwen3 {
    fn forward(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        cache: Option<ArcCacheItem>,
    ) -> Result<Array> {
        let x = &x;
        let shape = x.shape();
        let b = shape[0];
        let l = shape[1];

        let mut queries = self.q_proj.forward(x)?;
        let mut keys = self.k_proj.forward(x)?;
        let mut values = self.v_proj.forward(x)?;

        queries = self.q_norm.forward(
            &queries
                .reshape(&[b, l, self.n_heads, -1])?
                .transpose_axes(&[0, 2, 1, 3])?,
        )?;
        keys = self.k_norm.forward(
            &keys
                .reshape(&[b, l, self.n_kv_heads, -1])?
                .transpose_axes(&[0, 2, 1, 3])?,
        )?;
        values = values
            .reshape(&[b, l, self.n_kv_heads, -1])?
            .transpose_axes(&[0, 2, 1, 3])?;

        let mut maybe_cache_ref = cache.as_ref();

        if let Some(ref mut cache_ref) = maybe_cache_ref {
            let context = "reading cache for offset";
            let offset = cache_ref.read_lock(context)?.offset;
            queries = self.rope.forward(&queries, offset)?;
            keys = self.rope.forward(&keys, offset)?;

            let context = "updating cache";
            let (k, v) = cache_ref
                .write_lock(context)?
                .update_and_fetch(&keys, &values)?;
            keys = k;
            values = v;
        } else {
            queries = self.rope.forward(&queries, 0)?;
            keys = self.rope.forward(&keys, 0)?;
        }

        let mut output =
            scaled_dot_product_attention(&queries, &keys, &values, None, self.scale as f32, mask)?;

        output = output.transpose_axes(&[0, 2, 1, 3])?.reshape(&[b, l, -1])?;

        let output = self.o_proj.forward(&output)?;
        Ok(output)
    }

    fn set_weight(&mut self, name: &str, sub_name: &str, tensor: &Tensor) -> Result<()> {
        match sub_name {
            "q_proj.weight" => Ok(self.q_proj.update_weight(&tensor.data)),
            "k_proj.weight" => Ok(self.k_proj.update_weight(&tensor.data)),
            "v_proj.weight" => Ok(self.v_proj.update_weight(&tensor.data)),
            "o_proj.weight" => Ok(self.o_proj.update_weight(&tensor.data)),

            "q_proj.scales" => Ok(self.q_proj.update_scales(&tensor.data)),
            "k_proj.scales" => Ok(self.k_proj.update_scales(&tensor.data)),
            "v_proj.scales" => Ok(self.v_proj.update_scales(&tensor.data)),
            "o_proj.scales" => Ok(self.o_proj.update_scales(&tensor.data)),

            "q_proj.biases" => Ok(self.q_proj.update_biases(&tensor.data)),
            "k_proj.biases" => Ok(self.k_proj.update_biases(&tensor.data)),
            "v_proj.biases" => Ok(self.v_proj.update_biases(&tensor.data)),
            "o_proj.biases" => Ok(self.o_proj.update_biases(&tensor.data)),

            "k_norm.weight" => Ok(self.k_norm.update_weight(&tensor.data)),
            "q_norm.weight" => Ok(self.q_norm.update_weight(&tensor.data)),
            _ => Err(Error::UnsupportedWeight(name.to_string())),
        }
    }
}

impl AttentionQwen3 {
    pub fn new(qwen3_config: Rc<Qwen3Config>) -> Result<AttentionQwen3> {
        let hidden_size = qwen3_config.hidden_size;
        let n_heads = qwen3_config.num_attention_heads;
        let n_kv_heads = qwen3_config.num_key_value_heads;
        let attention_bias = qwen3_config.attention_bias;

        if hidden_size % n_heads != 0 {
            return Err(Error::InvalidConfig(
                "hidden_size must be divisible by n_heads".into(),
            ));
        }

        if n_heads % n_kv_heads != 0 {
            return Err(Error::InvalidConfig(
                "n_heads must be divisible by n_kv_heads".into(),
            ));
        }

        let head_dim = qwen3_config.head_dim.unwrap_or(hidden_size / n_heads);
        let scale = 1.0 / (head_dim as f64).sqrt();

        let q_proj = MaybeQuantized::new(
            LinearBuilder {
                input_dims: hidden_size,
                output_dims: n_heads * head_dim,
                bias: attention_bias,
            }
            .build()?,
        );

        let k_proj = MaybeQuantized::new(
            LinearBuilder {
                input_dims: hidden_size,
                output_dims: n_kv_heads * head_dim,
                bias: attention_bias,
            }
            .build()?,
        );
        let v_proj = MaybeQuantized::new(
            LinearBuilder {
                input_dims: hidden_size,
                output_dims: n_kv_heads * head_dim,
                bias: attention_bias,
            }
            .build()?,
        );

        let o_proj = MaybeQuantized::new(
            LinearBuilder {
                input_dims: n_heads * head_dim,
                output_dims: hidden_size,
                bias: attention_bias,
            }
            .build()?,
        );

        let q_norm = RmsNormBuilder {
            dimensions: head_dim,
            eps: qwen3_config.rms_norm_eps,
        }
        .build()?;

        let k_norm = RmsNormBuilder {
            dimensions: head_dim,
            eps: qwen3_config.rms_norm_eps,
        }
        .build()?;

        let rope = RopeQwen3::new(head_dim, qwen3_config.rope_theta, false)?;

        Ok(AttentionQwen3 {
            n_heads,
            n_kv_heads,
            scale,
            q_proj,
            k_proj,
            v_proj,
            o_proj,
            rope,
            q_norm,
            k_norm,
        })
    }
}
