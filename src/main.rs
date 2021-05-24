#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

use egui_template::RsMark;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]

fn main() {
    let app = egui_template::RsMark::yolo();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
