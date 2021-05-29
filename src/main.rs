#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![warn(clippy::pedantic, rust_2018_idioms)]

use std::env;

use yolo_mark_rs::{wrangle_args, KeyboardMapping, RsMark};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]

fn main() {
    match wrangle_args(env::args()) {
        Ok(args) => {
            let app = RsMark::yolo(args, KeyboardMapping::default());
            let native_options = eframe::NativeOptions::default();
            eframe::run_native(Box::new(app), native_options);
        }
        Err(explanation) => {
            println!("{}", explanation)
        }
    }
}
