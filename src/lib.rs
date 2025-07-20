pub mod app;
pub mod core;
pub mod utils;

#[cfg(not(target_arch = "wasm32"))]
mod cli;
