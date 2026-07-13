//! WASM entry points exported to JavaScript.
//!
//! # Bootstrap sequence
//!
//! 1. Page loads wasm-bindgen glue and calls `await init()`.
//! 2. [`init`] — sets up `console_log` and a panic hook with backtrace.
//! 3. App entry — the app crate exports `main_run` (see [`web-example`](../../web-example/src/lib.rs))
//!    which runs the function marked with `#[cotis_start_async]`.

use cotis::launch::cotis_launch_async;
use log::error;
use std::backtrace::Backtrace;
use std::panic;
use wasm_bindgen::prelude::wasm_bindgen;

/// Initializes logging and a WASM panic hook. Call once after wasm-bindgen `init()`.
pub fn init() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    set_panic_hook();
}

/// Dispatches to the application entry hook registered by `#[cotis_start_async]`.
pub async fn main_run() {
    cotis_launch_async().await;
}

/// Installs a panic hook that logs the message and a captured backtrace to the console.
pub fn set_panic_hook() {
    panic::set_hook(Box::new(hook_impl));
}

fn hook_impl(info: &panic::PanicHookInfo) {
    let backtrace = Backtrace::force_capture();
    error!("Panic occurred: {info}\nBacktrace:\n{backtrace}");
}

#[wasm_bindgen]
pub fn init_wasm() {
    crate::start_up::init();
}

/// Legacy export name kept for older `index.html` bootstraps that scan `wasmEntry_*`.
#[wasm_bindgen]
#[allow(non_snake_case)]
pub async fn wasmEntry_main_run() {
    cotis_launch_async().await;
}
