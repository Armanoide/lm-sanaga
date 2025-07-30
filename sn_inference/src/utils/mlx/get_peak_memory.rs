use crate::error::{Error, Result};
use mlx_sys::mlx_get_peak_memory;

pub fn get_peak_memory() -> Result<usize> {
    let mut result: usize = 0;
    let code = unsafe { mlx_get_peak_memory(&mut result as *mut usize) };
    if code == 0 {
        Ok(result)
    } else {
        Err(Error::MemoryPeakQueryFailure)
    }
}
