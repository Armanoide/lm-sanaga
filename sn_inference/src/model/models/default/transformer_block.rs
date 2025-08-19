#[macro_export]
macro_rules! default_forward_transformer_block {
    (
        $self:ident,
        $x:expr,
        $mask:expr,
        $cache:expr
    ) => {{
        let normed_input = $self.input_layernorm.forward($x)?;
        let attn_output = $self.self_attn.forward(&normed_input, $mask, $cache)?;
        let residual = $x + attn_output;
        let normed_residual = $self.post_attention_layernorm.forward(&residual)?;
        let mlp_output = $self.mlp.forward(&normed_residual, $mask, None)?;
        Ok(residual + mlp_output)
    }};
}
