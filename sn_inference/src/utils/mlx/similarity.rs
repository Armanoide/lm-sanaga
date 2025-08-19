use crate::error::Result;
use mlx_rs::Array;

pub fn similarity_cos(a: &Array, b: &Array) -> Result<Array> {
    // Assume a and b shapes: [1, embedding_dim]
    let b = b.transpose()?;
    let sim_matrix = a.matmul(&b)?;
    sim_matrix.eval()?;
    Ok(sim_matrix)
}
