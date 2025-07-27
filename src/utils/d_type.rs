use mlx_rs::Dtype;

pub trait DTypeExt {
    fn from_string_unsafe(type_in_str: &str) -> Dtype;
}

impl DTypeExt for Dtype {
    fn from_string_unsafe(type_in_str: &str) -> Dtype {
        match type_in_str.to_lowercase().as_str() {
            "bool" => Dtype::Bool,
            "u8" => Dtype::Uint8,
            "u16" => Dtype::Uint16,
            "u32" => Dtype::Uint32,
            "u64" => Dtype::Uint64,
            "i8" => Dtype::Int8,
            "i16" => Dtype::Int16,
            "i32" => Dtype::Int32,
            "i64" => Dtype::Int64,
            "f16" => Dtype::Float16,
            "f32" => Dtype::Float32,
            "f64" => Dtype::Float64,
            "bf16" => Dtype::Bfloat16,
            "bfloat16" => Dtype::Bfloat16,
            "Complex64" => Dtype::Complex64,
            _ => Dtype::Bool,
        }
    }
}
