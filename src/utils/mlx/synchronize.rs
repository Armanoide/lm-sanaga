use crate::error::{Error, Result};
use mlx_rs::Stream;
use mlx_sys::mlx_set_wired_limit;

pub fn synchronize(stream: Option<Stream>) -> Result<(usize)> {
    Ok(0)
    /*let mut result: usize = 0;
    let code = unsafe { mlx_set_wired_limit(&mut result as *mut usize, limit as usize) };
    if code == 0 {
        Ok(result)
    } else {
        Err(Error::MlxFunctionLoadFailure(
            "Failed to check if Metal is available".to_string(),
        ))
    }*/
}
