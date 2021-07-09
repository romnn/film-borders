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

// #[derive(Serialize, Deserialize, Clone, Debug)]

pub struct FilmImage {
    // pixels: Vec<u8>,
    // img: DynamicImage,
    pub buffer: RgbaImage,
    pub file_path: Option<PathBuf>,
    pub size: Size,
}

// impl FilmImage {
//     pub fn dyn_image_from_raw(: &PhotonImage) -> DynamicImage {
//         // convert a vec of raw pixels (as u8s) to a DynamicImage type
//     }
// }

// pub fn watermark(mut img: &mut PhotonImage, watermark: &PhotonImage, x: u32, y: u32) {
//     let dyn_watermark: DynamicImage = crate::helpers::dyn_image_from_raw(&watermark);
//     let mut dyn_img: DynamicImage = crate::helpers::dyn_image_from_raw(&img);
//     image::imageops::overlay(&mut dyn_img, &dyn_watermark, x, y);
//     img.raw_pixels = dyn_img.to_bytes();
// }

// pub fn to_raw_pixels(imgdata: ImageData) -> Vec<u8> {
//     imgdata.data().to_vec()
// }

// todo: abstract class that operates on the image image type and does all of the processing
// todo: one wrapper for the filesystem that loads and saves to disk
// todo: one wrapper that reads from canvas and returns raw pixels

fn get_image_data(
    canvas: &HtmlCanvasElement,
    ctx: &CanvasRenderingContext2d,
) -> Result<DynamicImage, JsValue> {
    let width = canvas.width();
    let height = canvas.height();
    // let data: ImageData = ctx.get_image_data(0.0, 0.0, 100.0, 100.0);
    let pixels = ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)?
        // .map_err(|err: JsValue| {
        //     ImageError::IoError(IOError::new(
        //         ErrorKind::Other,
        //         err.as_string()
        //             .unwrap_or(String::from("failed to read image data from canvas.")),
        //     ))
        // })?
        .data()
        .to_vec();
    // let pixels = data
    // let _len_vec = photon_image.raw_pixels.len() as u128;
    // let raw_pixels = &photon_image.raw_pixels;
    let img_buffer = ImageBuffer::from_vec(width, height, pixels)
        .ok_or_else(|| JsValue::from_str("failed to create ImageBuffer"))?;
    /*.ok_or_else(|| {
    ImageError::Decoding(DecodingError::new(
        ImageFormatHint::Unknown,
        IOError::new(ErrorKind::Other, "nooo"),
    ))*/
    // })?;
    // .unwrap();
    Ok(DynamicImage::ImageRgba8(img_buffer))

    // let _vec_data = data.data().to_vec();
    // data
}

// impl From<ImageError> for JsValue {
//     fn from(a: ImageError) -> JsValue {
//         JsValue {}
//     }
// }

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl FilmImage {
    // pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> FilmImage {
    //     FilmImage {
    //         pixels,
    //         width,
    //         height,
    //     }
    // }

    pub fn from_canvas(
        canvas: &HtmlCanvasElement,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<FilmImage, JsValue> {
        let buffer = get_image_data(&canvas, &ctx)?.to_rgba8();
        let width = buffer.width();
        let height = buffer.height();
        Ok(FilmImage {
            // img,
            buffer: buffer,
            file_path: None,
            size: Size {
                width: width,
                height: height,
            },
        })
        // let raw_pixels = to_raw_pixels(imgdata);
        // match get_image_data(&canvas, &ctx) {
        //     Ok(img) => Ok(FilmImage {
        //         img,
        //         file_path: None,
        //     }),
        //     Err(err) => Err(),
        // }
    }

    pub fn from_file(image_path: PathBuf) -> Result<FilmImage, ImageError> {
        let buffer = ImageReader::open(image_path.to_owned())?
            .decode()?
            .to_rgba8();
        // let buffer = img.to_rgba8();
        let width = buffer.width();
        let height = buffer.height();
        Ok(FilmImage {
            // img: img.clone(),
            buffer: buffer,
            file_path: Some(image_path),
            size: Size {
                width: width,
                height: height,
            },
        })
        // Ok(ImageBorders {
        //     image_path: image_path,
        //     output_path: default_output,
        //     img: img.clone(),
        // })
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

    pub fn fade_out(
        buffer: &mut RgbaImage,
        // iter: std::iter::Iterator<Item=u32>,
        start: u32,
        end: u32,
        direction: Direction,
    ) -> () {
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

                // buffer.put_pixel(x, y, Rgba.from_slice(channels));
                // pixel.map(|channels| {
                //     // channels.0 = 255;
                //     channels
                // });
                // pixel.apply_with_alpha(
                //     |channels| channels,
                //     |alpha| ,
                // );
                // buffer.put_pixel(x, y, (*buffer.get_pixel(x, y)).blend(color));
                // buffer.get_pixel_mut(x, y).blend(&color);
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
                // buffer.put_pixel(x, y, (*buffer.get_pixel(x, y)).blend(color));
                buffer.get_pixel_mut(x, y).blend(&color);
            }
        }
    }

    // pub fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> Result<(), ImageError> {
    pub fn crop(
        // buffer: &mut RgbaImage,
        buffer: RgbaImage,
        // buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
        top_left: Point,
        bottom_right: Point,
        // ) -> Result<SubImage<&mut DynamicImage>, ImageError> {
        // ) -> Result<(), ImageError> {
    ) -> Result<RgbaImage, ImageError> {
        let x = top_left.x;
        let y = top_left.y;
        let width = (bottom_right.x as i64 - x as i64).abs() as u32;
        let height = (bottom_right.y as i64 - y as i64).abs() as u32;
        // self.img = DynamicImage::ImageRgba8(crop(&mut self.img, x, y, width, height).to_image());
        let mut buffer2 = buffer.clone();
        Ok(crop(&mut buffer2, x, y, width, height).to_image())
        // Ok(crop(&mut self.img, x, y, width, height))
    }

    // pub fn overlay(
    //     background: &mut RgbaImage,
    //     buffer: &RgbaImage,
    //     x: u32,
    //     y: u32,
    // ) -> Result<(), ImageError> {
    //     // self.img = DynamicImage::ImageRgba8(resize(&self.img, width, height, FilterType::Lanczos3));
    //     // Ok(DynamicImage::ImageRgba8(resize(&self.img, width, height, FilterType::Lanczos3)))
    //     // self.img = DynamicImage::ImageRgba8(resize(&self.img, width, height, FilterType::Lanczos3));
    //     Ok(overlay(&mut background, &buffer, x, y))
    // }

    pub fn resize(buffer: RgbaImage, width: u32, height: u32) -> Result<RgbaImage, ImageError> {
        // self.img = DynamicImage::ImageRgba8(resize(&self.img, width, height, FilterType::Lanczos3));
        // Ok(DynamicImage::ImageRgba8(resize(&self.img, width, height, FilterType::Lanczos3)))
        // self.img = DynamicImage::ImageRgba8(resize(&self.img, width, height, FilterType::Lanczos3));
        Ok(resize(&buffer, width, height, FilterType::Lanczos3))
    }
}
