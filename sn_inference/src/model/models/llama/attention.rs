use crate::cache::k_v_cache::ArcCacheItem;
use crate::config::config_model::ConfigModel;
use crate::config::config_models::llama::LLaMAConfig;
use crate::error::{Error, Result};
use crate::factory::rope::initialize_rope;
use crate::mask::mask::AttentionMask;
use crate::model::models::llama::rope::RopeLlama;
use crate::model::weight::Tensor;
use crate::module::Module;
use crate::quantized::Quantize;
use crate::utils::maybe_quantized::MaybeQuantizedLinear;
use crate::utils::scaled_dot_product_attention::scaled_dot_product_attention;
use mlx_rs::Array;
use mlx_rs::builder::Builder;
use mlx_rs::module::Module as MLXModule;
use mlx_rs::nn::{Linear, LinearBuilder};
use mlx_rs::quantization::{MaybeQuantized, Quantizable};
use sn_core::utils::rw_lock::{RwLockExt, RwLockExtOpt};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct AttentionLlama {
    hidden_size: i32,
    n_heads: i32,
    n_kv_heads: i32,
    head_dim: i32,
    scale: f64,

    q_proj: MaybeQuantized<Linear>,
    k_proj: MaybeQuantized<Linear>,
    v_proj: MaybeQuantized<Linear>,
    o_proj: MaybeQuantized<Linear>,
    rope: RopeLlama,
}

impl Quantize for AttentionLlama {
    fn quantize(&mut self, group_size: i32, bits: i32) -> Result<()> {
        self.q_proj = self.q_proj.clone().try_into_quantized(group_size, bits)?;
        self.k_proj = self.k_proj.clone().try_into_quantized(group_size, bits)?;
        self.v_proj = self.v_proj.clone().try_into_quantized(group_size, bits)?;
        self.o_proj = self.o_proj.clone().try_into_quantized(group_size, bits)?;
        Ok(())
    }
}

impl Module for AttentionLlama {
    fn forward(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        cache: Option<ArcCacheItem>,
    ) -> Result<Array> {
        let shape = x.shape();
        let b = shape[0];
        let l = shape[1];

        let mut queries = self.q_proj.forward(x)?;
        let mut keys = self.k_proj.forward(x)?;
        let mut values = self.v_proj.forward(x)?;

        // Prepare the queries, keys and values for the attention computation
        queries = queries
            .reshape(&[b, l, self.n_heads, -1])?
            .transpose_axes(&[0, 2, 1, 3])?;
        keys = keys
            .reshape(&[b, l, self.n_kv_heads, -1])?
            .transpose_axes(&[0, 2, 1, 3])?;
        values = values
            .reshape(&[b, l, self.n_kv_heads, -1])?
            .transpose_axes(&[0, 2, 1, 3])?;

        let mut maybe_cache_ref = cache.as_ref(); //.map(|rc| rc);

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

        let output = scaled_dot_product_attention(
            &queries,
            &keys,
            &values,
            maybe_cache_ref.read_lock_mut("")?.as_deref(),
            self.scale as f32,
            mask,
        )?;

        let output = output.transpose_axes(&[0, 2, 1, 3])?.reshape(&[b, l, -1])?;
        //panic!();
        Ok(self.o_proj.forward(&output)?)
    }

    fn set_weight(&mut self, name: &str, tensor: &Tensor) -> Result<()> {
        if let Some(layer_without_suffix) = name.splitn(5, '.').nth(4) {
            match layer_without_suffix {
                "q_proj.weight" => self.q_proj.update_weight(&tensor.data),
                "k_proj.weight" => self.k_proj.update_weight(&tensor.data),
                "v_proj.weight" => self.v_proj.update_weight(&tensor.data),
                "o_proj.weight" => self.o_proj.update_weight(&tensor.data),

                "q_proj.scales" => self.q_proj.update_scales(&tensor.data),
                "k_proj.scales" => self.k_proj.update_scales(&tensor.data),
                "v_proj.scales" => self.v_proj.update_scales(&tensor.data),
                "o_proj.scales" => self.o_proj.update_scales(&tensor.data),

                "q_proj.biases" => self.q_proj.update_biases(&tensor.data),
                "k_proj.biases" => self.k_proj.update_biases(&tensor.data),
                "v_proj.biases" => self.v_proj.update_biases(&tensor.data),
                "o_proj.biases" => self.o_proj.update_biases(&tensor.data),

                _ => return Err(Error::UnsupportedWeight(name.to_string())),
            }
        }
        Ok(())
    }
}

impl AttentionLlama {
    pub fn new(llama_config: Rc<LLaMAConfig>) -> Result<AttentionLlama> {
        let hidden_size = llama_config.hidden_size;
        let n_heads = llama_config.num_attention_heads;
        let n_kv_heads = llama_config.num_key_value_heads;
        let attention_bias = llama_config.attention_bias;

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

        let head_dim = hidden_size / n_heads;
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

        let scaling_config = &llama_config.rope_scaling;

        let rope = initialize_rope(
            head_dim,
            llama_config.rope_theta,
            false,
            ConfigModel::LLaMA(llama_config),
        )?;

        Ok(AttentionLlama {
            hidden_size,
            n_heads,
            n_kv_heads,
            head_dim,
            scale,
            q_proj,
            k_proj,
            v_proj,
            o_proj,
            rope,
        })
    }
}
