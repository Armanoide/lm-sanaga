use crate::error::{Error, Result};
use mlx_sys::mlx_set_wired_limit;

pub fn set_wired_limit(limit: usize) -> Result<(usize)> {
    let mut result: usize = 0;
    let code = unsafe { mlx_set_wired_limit(&mut result as *mut usize, limit as usize) };
    if code == 0 {
        Ok(result)
    } else {
        Err(Error::MlxFunctionLoadFailure(
            "Failed to check set weird limit with Metal".to_string(),
        ))
    }
}
