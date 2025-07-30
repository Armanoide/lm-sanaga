use crate::error::Error;

pub trait Quantize {
    fn quantize(&mut self, group_size: i32, bits: i32) -> Result<(), Error>;
}
