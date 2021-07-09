use crate::borders::{Point, Size};
use crate::utils;
use image::codecs::jpeg::JpegEncoder;
use image::error::{DecodingError, ImageError, ImageFormatHint, ImageResult};
use image::imageops::{crop, overlay, resize, FilterType};
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer, Pixel, Rgba, RgbaImage, SubImage};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min, PartialOrd};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Error as IOError, ErrorKind};
use std::path::{Path, PathBuf};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

pub struct FilmImage {
    pub buffer: RgbaImage,
    pub file_path: Option<PathBuf>,
    pub size: Size,
}
fn get_image_data(
    canvas: &HtmlCanvasElement,
    ctx: &CanvasRenderingContext2d,
) -> Result<DynamicImage, JsValue> {
    let width = canvas.width();
    let height = canvas.height();
    let pixels = ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)?
        .data()
        .to_vec();
    let img_buffer = ImageBuffer::from_vec(width, height, pixels)
        .ok_or_else(|| JsValue::from_str("failed to create ImageBuffer"))?;
    Ok(DynamicImage::ImageRgba8(img_buffer))
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl FilmImage {
    pub fn from_canvas(
        canvas: &HtmlCanvasElement,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<FilmImage, JsValue> {
        let buffer = get_image_data(&canvas, &ctx)?.to_rgba8();
        let width = buffer.width();
        let height = buffer.height();
        Ok(FilmImage {
            buffer: buffer,
            file_path: None,
            size: Size {
                width: width,
                height: height,
            },
        })
    }

    pub fn from_file(image_path: PathBuf) -> Result<FilmImage, ImageError> {
        let buffer = ImageReader::open(image_path.to_owned())?
            .decode()?
            .to_rgba8();
        let width = buffer.width();
        let height = buffer.height();
        Ok(FilmImage {
            buffer: buffer,
            file_path: Some(image_path),
            size: Size {
                width: width,
                height: height,
            },
        })
    }

    fn get_output_path(&self, output_path: Option<String>) -> Result<String, ImageError> {
        let base_dir = (self
            .file_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf()))
        .or_else(|| env::current_dir().ok());
        let default_output = base_dir.and_then(|b| {
            match self
                .file_path
                .as_ref()
                .and_then(|name| name.file_stem())
                .and_then(|name| name.to_str())
            {
                Some(stem) => Some(b.join(format!("{}_with_border.png", stem))),
                None => None,
            }
        });

        let default_output = default_output.and_then(|p| p.into_os_string().into_string().ok());
        output_path
            .or(default_output)
            .ok_or(ImageError::IoError(IOError::new(ErrorKind::Other, "nooo")))
    }

    pub fn save_jpeg_to_file(
        &self,
        buffer: RgbaImage,
        output_path: Option<String>,
        quality: Option<u8>,
    ) -> Result<(), ImageError> {
        let output_path = self.get_output_path(output_path)?;
        println!("saving to {}...", output_path);
        let mut file = File::create(&output_path)?;
        let mut encoder = JpegEncoder::new_with_quality(&mut file, quality.unwrap_or(80));
        encoder.encode_image(&DynamicImage::ImageRgba8(buffer.clone()));
        Ok(())
    }

    pub fn save_to_file(
        &self,
        buffer: RgbaImage,
        output_path: Option<String>,
    ) -> Result<(), ImageError> {
        let output_path = self.get_output_path(output_path)?;
        println!("saving to {}...", output_path);
        DynamicImage::ImageRgba8(buffer.clone()).save(&output_path)?;
        Ok(())
    }

    pub fn fade_out(buffer: &mut RgbaImage, start: u32, end: u32, direction: Direction) -> () {
        let other = match direction {
            Direction::Horizontal => buffer.height(),
            Direction::Vertical => buffer.width(),
        };
        let diff = (end as f32 - start as f32).abs();
        for i in min(start, end)..=max(start, end) {
            let ir = i - min(start, end);
            let mut frac = ir as f32 / diff;
            if start < end {
                frac = 1.0 - frac;
            }
            let alpha = (255.0 * frac) as u8;
            // println!("alpha = {} = {} / {}", alpha, ir, range);
            for j in 0..other {
                let (x, y) = match direction {
                    Direction::Horizontal => (i, j),
                    Direction::Vertical => (j, i),
                };
                let channels = buffer.get_pixel_mut(x, y).channels_mut();
                channels[3] = min(channels[3], alpha);
            }
        }
    }

    pub fn fill_rect(
        buffer: &mut RgbaImage,
        color: Rgba<u8>,
        top_left: Point,
        bottom_right: Point,
    ) -> () {
        let x1 = utils::clamp(min(top_left.x, bottom_right.x), 0, buffer.width());
        let x2 = utils::clamp(max(top_left.x, bottom_right.x), 0, buffer.width());
        let y1 = utils::clamp(min(top_left.y, bottom_right.y), 0, buffer.height());
        let y2 = utils::clamp(max(top_left.y, bottom_right.y), 0, buffer.height());
        for x in x1..x2 {
            for y in y1..y2 {
                buffer.get_pixel_mut(x, y).blend(&color);
            }
        }
    }
}
