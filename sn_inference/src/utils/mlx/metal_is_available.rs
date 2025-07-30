use crate::error::{Error, Result};
use mlx_sys::mlx_metal_is_available;

pub fn metal_is_available() -> Result<bool> {
    let mut result: bool = false;
    let code = unsafe { mlx_metal_is_available(&mut result as *mut bool) };
    if code == 0 {
        Ok(result)
    } else {
        Err(Error::MlxFunctionLoadFailure(
            "Failed to check if Metal is available".to_string(),
        ))
    }
}
