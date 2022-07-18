#![forbid(unsafe_code)]
#![warn(clippy::pedantic, rust_2018_idioms)]

pub use app::arguments::wrangle_args;
pub use app::keyboard_mapping::KeyboardMapping;
pub use app::RsMark;

mod app;
