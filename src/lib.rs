#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod watermark;
mod image_ops;
mod web_file;

pub use app::WatermarkApp;
pub use watermark::{Position, WatermarkConfig};
pub use image_ops::{add_watermark, load_image_from_bytes, save_image_as_png, save_image_as_jpg, get_image_info};
pub use web_file::WebFileReader;
