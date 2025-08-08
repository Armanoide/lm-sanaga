use crate::error::{Error, Result};
use mlx_rs::{Array, rope};
use crate::model::models::default::rope::BaseRope;

const PI: f32 = std::f64::consts::PI as f32;
#[derive(Clone, Debug)]
pub struct RopeQwen3 {
    dims: i32,
    traditional: bool,
    base: Option<f32>,
}

impl RopeQwen3 {
    pub fn new(
        dims: i32,
        base: f32,
        traditional: bool,
    ) -> Result<RopeQwen3> {
        let dims = dims;
        let traditional = traditional;

        Ok(RopeQwen3 {
            dims,
            traditional,
            base: Some(base),
        })
    }

    pub fn forward(&self, x: &Array, offset: i32) -> Result<Array> {
        rope!(
            array = x,
            dimensions = self.dims,
            traditional = self.traditional,
            base = self.base,
            scale = 1.0,
            offset = offset
        )
        .map_err(|e| Error::ExceptionMLX(e))
    }
}

impl BaseRope for RopeQwen3 {

}