
/// A global mutex to **serialize access to MLX GPU compute**.
///
/// MLX/Metal does **not support concurrent compute operations**
/// (e.g., async_eval / eval), and doing so can cause
/// segmentation faults or GPU command buffer assertion failures.
///
/// This lock ensures that **only one thread/task** runs
/// compute operations like `async_eval` at a time.
pub static MLX_COMPUTE_LOCK: once_cell::sync::Lazy<std::sync::Mutex<bool>> = once_cell::sync::Lazy::new(|| {
    std::sync::Mutex::new(true)
});
