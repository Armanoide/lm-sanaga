use crate::error::{Error, Result};
use mlx_sys::mlx_metal_device_info;
use std::{ffi::CStr, os::raw::c_char};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MetalDeviceInfoRaw {
    architecture: [c_char; 256],
    max_buffer_length: usize,
    max_recommended_working_set_size: usize,
    memory_size: usize,
}

/// Safe Rust representation of Metal device information.
#[derive(Debug, Clone)]
pub struct MetalDeviceInfo {
    pub architecture: String,
    pub max_buffer_length: usize,
    pub max_recommended_working_set_size: usize,
    pub memory_size: usize,
}

impl TryFrom<MetalDeviceInfoRaw> for MetalDeviceInfo {
    type Error = Error;

    fn try_from(raw: MetalDeviceInfoRaw) -> Result<MetalDeviceInfo> {
        // Convert C char array to Rust String safely
        let arch_cstr = unsafe { CStr::from_ptr(raw.architecture.as_ptr()) };
        let architecture = arch_cstr.to_str()?.to_owned();

        Ok(Self {
            architecture,
            max_buffer_length: raw.max_buffer_length,
            max_recommended_working_set_size: raw.max_recommended_working_set_size,
            memory_size: raw.memory_size,
        })
    }
}

pub fn metal_device_info() -> Result<MetalDeviceInfo> {
    let raw = unsafe { mlx_metal_device_info() };

    let raw_info: MetalDeviceInfoRaw =
        unsafe { std::ptr::read(&raw as *const _ as *const MetalDeviceInfoRaw) };

    MetalDeviceInfo::try_from(raw_info).map_err(|e| Error::MlxFunctionLoadFailure(e.to_string()))
}
