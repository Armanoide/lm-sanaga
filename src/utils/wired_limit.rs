use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use mlx_sys::{mlx_metal_device_info, mlx_metal_is_available};
use crate::model::model_kind::ModelKind;

pub fn wired_limit(model: Arc<RwLock<ModelKind>>) {
    //mlx_metal_device_info()
    //mlx_metal_is_available()
}