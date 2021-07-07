use crate::img::FilmImage;
use crate::utils;
use image::error::{DecodingError, ImageError, ImageFormatHint, ImageResult};
use image::imageops::{crop, overlay, resize, FilterType};
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer, Pixel, Rgba, RgbaImage};
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
    pub width: u32,
    pub height: u32,
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
    pub x: u32,
    pub y: u32,
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
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
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
    pub scale_factor: Option<f32>,
    pub crop: Option<Crop>,
    pub border_width: Option<Sides>,
    // pub padding: Option<Sides>,
    pub rotate_angle: Option<i16>,
}

pub struct ImageBorders {
    img: FilmImage,
}

impl ImageBorders {
    pub fn new(img: FilmImage) -> ImageBorders {
        utils::set_panic_hook();
        ImageBorders { img }
    }

    pub fn save_jpeg(
        &self,
        buffer: RgbaImage,
        output_path: Option<String>,
        quality: Option<u8>,
    ) -> Result<(), ImageError> {
        self.img.save_jpeg_to_file(buffer, output_path, quality)
    }

    pub fn save(&self, buffer: RgbaImage, output_path: Option<String>) -> Result<(), ImageError> {
        self.img.save_to_file(buffer, output_path)
    }

    pub fn apply(&mut self, options: ImageBorderOptions) -> Result<RgbaImage, ImageError> {
        // let size = self.img.size;
        let mut size = Size {
            width: self.img.buffer.width(),
            height: self.img.buffer.height(),
        };
        if let Some(output_size) = options.output_size {
            size = output_size
        };
        let mut final_image = RgbaImage::new(size.width, size.height);

        // fill white
        let white_color = Rgba::from_channels(255, 255, 255, 255);
        FilmImage::fill_rect(
            &mut final_image,
            white_color,
            Point { x: 0, y: 0 },
            Point {
                x: size.width,
                y: size.height,
            },
        );

        // crop the image
        if let Some(crop_options) = options.crop {
            let crop_x = crop_options.top_left.x;
            let crop_y = crop_options.top_left.y;
            let crop_width = (crop_options.bottom_right.x as i64 - crop_x as i64).abs() as u32;
            let crop_height = (crop_options.bottom_right.y as i64 - crop_y as i64).abs() as u32;
            self.img.buffer = crop(
                &mut self.img.buffer,
                crop_x,
                crop_y,
                crop_width,
                crop_height,
            )
            .to_image()
        };

        // resize the image to fit the screen
        let (mut fit_width, mut fit_height) = utils::resize_dimensions(
            self.img.buffer.width(),
            self.img.buffer.height(),
            size.width,
            size.height,
            false,
        );
        println!("fitting to {} x {}", fit_width, fit_height);

        if let Some(scale_factor) = options.scale_factor {
            // scale the image by factor
            fit_width = (fit_width as f32 * utils::clamp(scale_factor, 0f32, 1f32)) as u32;
            fit_height = (fit_height as f32 * utils::clamp(scale_factor, 0f32, 1f32)) as u32;
            println!("scaling to {} x {}", fit_width, fit_height);
        };

        let fitted_image = resize(
            &self.img.buffer,
            fit_width,
            fit_height,
            FilterType::Lanczos3,
        );

        let overlay_x = (size.width - fitted_image.width()) / 2;
        let overlay_y = (size.height - fitted_image.height()) / 2;
        println!("overlaying at {} {}", overlay_x, overlay_y);

        // create the black borders
        if let Some(border_width) = options.border_width {
            let black_color = Rgba::from_channels(0, 0, 0, 255);
            FilmImage::fill_rect(
                &mut final_image,
                black_color,
                Point {
                    x: overlay_x - border_width.left,
                    y: overlay_y - border_width.top,
                },
                Point {
                    x: overlay_x + fitted_image.width() + border_width.right,
                    y: overlay_y + fitted_image.height() + border_width.bottom,
                },
            );
        };

        overlay(&mut final_image, &fitted_image, overlay_x, overlay_y);

        Ok(final_image)
    }

    pub fn from_file(input_path: PathBuf) -> Result<ImageBorders, ImageError> {
        let img = FilmImage::from_file(input_path)?;
        Ok(ImageBorders { img })
    }
}
