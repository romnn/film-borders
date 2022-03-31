use crate::borders::{Point, Size};
use crate::utils;
use image::codecs::jpeg::JpegEncoder;
use image::error::ImageError;
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer, Pixel, Rgba, RgbaImage};
use std::cmp::{max, min};
use std::env;
use std::fs::File;
use std::io::{Error as IOError, ErrorKind};
use std::path::{Path, PathBuf};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[inline]
pub fn fill_rect(buffer: &mut RgbaImage, color: &Rgba<u8>, top_left: Point, bottom_right: Point) {
    let x1 = utils::clamp(min(top_left.x, bottom_right.x), 0, buffer.width());
    let x2 = utils::clamp(max(top_left.x, bottom_right.x), 0, buffer.width());
    let y1 = utils::clamp(min(top_left.y, bottom_right.y), 0, buffer.height());
    let y2 = utils::clamp(max(top_left.y, bottom_right.y), 0, buffer.height());
    for x in x1..x2 {
        for y in y1..y2 {
            buffer.get_pixel_mut(x, y).blend(color);
        }
    }
}

#[inline]
pub fn fade_out(buffer: &mut RgbaImage, start: u32, end: u32, direction: Direction) {
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

pub struct Image {
    pub buffer: RgbaImage,
    pub file_path: Option<PathBuf>,
    pub size: Size,
}

#[inline]
fn image_from_image_data(img: ImageData) -> Result<DynamicImage, JsValue> {
    let pixels = img.data().to_vec();
    let img_buffer = ImageBuffer::from_vec(img.width(), img.height(), pixels)
        .ok_or_else(|| JsValue::from_str("failed to create ImageBuffer"))?;
    Ok(DynamicImage::ImageRgba8(img_buffer))
}

#[inline]
fn image_from_canvas(
    canvas: &HtmlCanvasElement,
    ctx: &CanvasRenderingContext2d,
) -> Result<DynamicImage, JsValue> {
    let width = canvas.width();
    let height = canvas.height();
    let img = ctx.get_image_data(0.0, 0.0, width as f64, height as f64)?;
    image_from_image_data(img)
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Image {
    #[allow(dead_code)]
    pub fn from_canvas(
        canvas: &HtmlCanvasElement,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<Image, JsValue> {
        let buffer = image_from_canvas(canvas, ctx)?.to_rgba8();
        let width = buffer.width();
        let height = buffer.height();
        Ok(Image {
            buffer,
            file_path: None,
            size: Size { width, height },
        })
    }

    #[allow(dead_code)]
    pub fn from_image_data(data: ImageData) -> Result<Self, JsValue> {
        let buffer = image_from_image_data(data)?.to_rgba8();
        let width = buffer.width();
        let height = buffer.height();
        Ok(Self {
            buffer,
            file_path: None,
            size: Size { width, height },
        })
    }

    pub fn from_file(image_path: &Path) -> Result<Self, ImageError> {
        let buffer = ImageReader::open(image_path)?.decode()?.to_rgba8();
        let width = buffer.width();
        let height = buffer.height();
        Ok(Self {
            buffer,
            file_path: Some(image_path.to_path_buf()),
            size: Size { width, height },
        })
    }

    fn get_output_path(&self, output_path: Option<PathBuf>) -> Result<PathBuf, ImageError> {
        let base_dir = (self
            .file_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf()))
        .or_else(|| env::current_dir().ok());
        let default_output = base_dir.and_then(|b| {
            self.file_path
                .as_ref()
                .and_then(|name| name.file_stem())
                .and_then(|name| name.to_str())
                .map(|stem| b.join(format!("{}_with_border.png", stem)))
        });

        output_path
            .or(default_output)
            .ok_or_else(|| ImageError::IoError(IOError::new(ErrorKind::Other, "nooo")))
    }

    #[allow(dead_code)]
    pub fn save_jpeg_to_file(
        &self,
        buffer: RgbaImage,
        output_path: Option<PathBuf>,
        quality: Option<u8>,
    ) -> Result<(), ImageError> {
        let output_path = self.get_output_path(output_path)?;
        println!("saving to {}...", output_path.display());
        let mut file = File::create(&output_path)?;
        let mut encoder = JpegEncoder::new_with_quality(&mut file, quality.unwrap_or(80));
        encoder.encode_image(&DynamicImage::ImageRgba8(buffer))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_to_file(
        &self,
        buffer: RgbaImage,
        output_path: Option<PathBuf>,
    ) -> Result<(), ImageError> {
        let output_path = self.get_output_path(output_path)?;
        println!("saving to {}...", output_path.display());
        DynamicImage::ImageRgba8(buffer).save(&output_path)?;
        Ok(())
    }
}
