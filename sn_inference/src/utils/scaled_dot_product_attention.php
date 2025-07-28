use std::ffi::{c_char, c_int, c_void, CStr};
use std::mem::MaybeUninit;
use mlx_rs::{Array, Stream, StreamOrDevice};
use mlx_rs::error::Exception;
use mlx_rs::utils::IntoOption;
use mlx_sys::{_mlx_error, mlx_array, mlx_stream, mlx_vector_array, mlx_vector_array_append_value, mlx_vector_array_new};
use crate::cache::k_v_cache::KVCache;
use crate::mask::mask::AttentionMask;

const CAUSAL_MASK_MODE: &CStr = c"causal";
const DEFAULT_MASK_MODE: &CStr = c"";
pub(crate) struct VectorArray {
    c_vec: mlx_sys::mlx_vector_array,
}

pub type MlxErrorHandler = Option<
    unsafe extern "C" fn(msg: *const std::os::raw::c_char, data: *mut std::os::raw::c_void),
>;


unsafe extern "C" fn rust_error_handler(msg: *const c_char, _data: *mut c_void) {
    if !msg.is_null() {
        let message = CStr::from_ptr(msg).to_string_lossy();
        eprintln!("🛑 MLX ERROR: {}", message);
    }
}


impl VectorArray {
    pub(crate) fn as_ptr(&self) -> mlx_sys::mlx_vector_array {
        self.c_vec
    }

    pub(crate) unsafe fn from_ptr(c_vec: mlx_sys::mlx_vector_array) -> Self {
        Self { c_vec }
    }
}

pub fn scaled_dot_product_attention(
    queries: &Array,
    keys: &Array,
    values: &Array,
    _cache: Option<&KVCache>,  // unused here but kept for API compatibility
    scale: f32,
    mask: Option<&AttentionMask>,
) -> Result<Array, Exception> {
    unsafe {
        // Set the MLX error handler to get detailed error messages from the C library
        mlx_sys::mlx_set_error_handler(
            Some(rust_error_handler),
            std::ptr::null_mut(),
            None,
        );
    }

    // Prepare the result storage (uninitialized mlx_array)
    let mut res = MaybeUninit::<mlx_array>::uninit();
    let mut status: c_int;

    // Get the CPU stream for execution (or your preferred stream)
    let stream = Stream::gpu();

    // Prepare mask_mode and masks vector according to the mask passed
    let (mask_mode, masks) = if let Some(mask) = mask {
        // Convert your Rust mask(s) into a mlx_vector_array
        // Assume AttentionMask can convert to &[Array] or &Array internally
        // Here I assume you have a way to get mask arrays slice or single array
        let va = unsafe { VectorArray::from_ptr(mlx_sys::mlx_vector_array_new()) };

        match mask {
            AttentionMask::None => {
                (DEFAULT_MASK_MODE, va)
            }
            AttentionMask::MaskArray(array) => {
                let  arr = unsafe {
                    let v = mlx_vector_array_new();
                    mlx_vector_array_append_value(v, array.as_ptr());
                    VectorArray::from_ptr(v)
                };
                (DEFAULT_MASK_MODE, arr)
            }
            /*AttentionMask::MaskArray(arrays) => {
                let va = VectorArray::try_from_iter(arrays.iter())?;
                (DEFAULT_MASK_MODE, va)
            }*/
            AttentionMask::Causal => {
                // causal mask mode uses empty vector array
                let va = unsafe { VectorArray::from_ptr(mlx_sys::mlx_vector_array_new()) };
                (CAUSAL_MASK_MODE, va)
            }
        }
    } else {
        (DEFAULT_MASK_MODE,  unsafe { VectorArray::from_ptr(mlx_sys::mlx_vector_array_new()) })
    };

    // Call the C function
    status = unsafe {
        mlx_sys::mlx_fast_scaled_dot_product_attention(
            res.as_mut_ptr(),
            queries.as_ref().as_ptr(),
            keys.as_ref().as_ptr(),
            values.as_ref().as_ptr(),
            scale,
            mask_mode.as_ptr(),
            masks.as_ptr(), // Pass by value (mlx_vector_array is a struct, not a pointer)
            stream.as_ref().as_ptr(),
        )
    };

    // Free the vector array resources if needed
    unsafe { mlx_sys::mlx_vector_array_free(masks.c_vec); }

    // Check status and convert result
    if status != 0 {
        return Err(Exception::from("mlx_fast_scaled_dot_product_attention failed"));
    }

    let arr = unsafe {
        let res = res.assume_init();
        Array::from_ptr(res)
    };

    Ok(arr)
}


/*pub fn scaled_dot_product_attention(
    queries: &Array,
    keys: &Array,
    values: &Array,
    cache: Option<&KVCache>,
    scale: f32,
    mask: Option<&AttentionMask>,
) -> Result<Array, Exception> {

    unsafe {
        mlx_sys::mlx_set_error_handler(
            Some(rust_error_handler),
            std::ptr::null_mut(), // optional user data
            None, // optional destructor for data
        );
    }

    let stream = StreamOrDevice::default();
    if 2 == 3 {
        //todo: if instance of QuantizedKVCache
        /*
                return quantized_scaled_dot_product_attention(
                queries,
                keys,
                values,
                scale=scale,
                mask=mask,
                group_size=cache.group_size,
                bits=cache.bits,
            )
        */
        Err(Exception::from("no implemented"))
    } else {
        //if let Some(mask) = mask {
        //    return Ok(mlx_rs::fast::scaled_dot_product_attention(queries, keys, values, scale, mask.to_scaled_mask())?)
        //}
        // if let Some(mask) = mask {
        //    Ok(mlx_rs::fast::scaled_dot_product_attention(queries, keys, values, scale, mask.to_scaled_mask())?)
        //} else {
            //Ok(mlx_rs::fast::scaled_dot_product_attention(queries, keys, values, scale, None)?)
        let mut res = MaybeUninit::<mlx_array>::uninit();
        let mut status: c_int = 0;

        let masks = unsafe { mlx_vector_array_new() };

        //let result = unsafe { mlx_sys::mlx_vector_array_append_value(masks, DEFAULT_MASK_MODE) };

        //if result != 0 {
        //    panic!("Failed to append mask to vector array");
        //}
        //let masks = mlx_vector_array {
        //    ctx: std::ptr::null_mut(), // or real ctx if needed
        //};

            let mut cb = | _: () | unsafe {
                status = mlx_sys::mlx_fast_scaled_dot_product_attention(
                    res.as_mut_ptr(),
                    queries.as_ref().as_ptr(),
                    keys.as_ref().as_ptr(),
                    values.as_ref().as_ptr(),
                    scale,
                    DEFAULT_MASK_MODE.as_ptr(),
                    masks,//masks.as_ptr(),
                    stream.as_ref().as_ptr(),
                );
            };
            cb(());
            if status != 0 {
               return  Err(Exception::from("error :/"))
            }
            let arr = unsafe {
                let res = res.assume_init();
                Array::from_ptr(res)
            };
            Ok(arr)
        }
}*/
