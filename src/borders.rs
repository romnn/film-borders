use crate::img;
use crate::utils;
use image::error::{DecodingError, ImageError, ImageFormatHint, ImageResult};
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::io::{Error as IOError, ErrorKind};
use std::path::{Path, PathBuf};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[wasm_bindgen]
#[derive(Default, Copy, Clone)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

// impl Default for Size {
//     fn default() -> Self {
//         Size {
//             width: 0,
//             height: 0,
//         }
//     }
// }

#[wasm_bindgen]
#[derive(Default, Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
// impl Default for Point {
//     fn default() -> Self {
//         Point { x: 0, y: 0 }
//     }
// }

#[wasm_bindgen]
#[derive(Default, Copy, Clone)]
pub struct Crop {
    pub top_left: Point,
    pub bottom_right: Point,
}

// impl Default for Crop {
//     fn default() -> Self {
//         Crop {
//             top_left: 0,
//             bottom_right: 0,
//         }
//     }
// }

#[wasm_bindgen]
#[derive(Default, Copy, Clone)]
pub struct Sides {
    pub top: i32,
    pub left: i32,
    pub right: i32,
    pub bottom: i32,
}

// impl Default for Sides {
//     fn default() -> Self {
//         Sides {
//             top: 0,
//             left: 0,
//             right: 0,
//             bottom: 0,
//         }
//     }
// }

#[wasm_bindgen]
#[derive(Default, Copy, Clone)]
pub struct ImageBorderOptions {
    pub output_size: Option<Size>,
    pub crop: Option<Crop>,
    pub border_width: Option<Sides>,
    pub padding: Option<Sides>,
    pub rotate_angle: Option<i16>,
}

pub struct ImageBorders {
    img: img::FilmImage,
}

impl ImageBorders {
    pub fn new(img: img::FilmImage) -> ImageBorders {
        utils::set_panic_hook();
        ImageBorders { img }
    }

    pub fn save(&self, output_path: Option<String>, quality: Option<u8>) -> Result<(), ImageError> {
        self.img.save_to_file(output_path, quality)
    }

    pub fn apply(&self, options: ImageBorderOptions) -> Result<(), ImageError> {
        Ok(())
    }

    pub fn from_file(input_path: PathBuf) -> Result<ImageBorders, ImageError> {
        let img = img::FilmImage::from_file(input_path)?;
        Ok(ImageBorders { img })
    }
}
