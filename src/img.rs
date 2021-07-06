use crate::utils;
use image::codecs::jpeg::JpegEncoder;
use image::error::{DecodingError, ImageError, ImageFormatHint, ImageResult};
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer};
use serde::{Deserialize, Serialize};
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
    img: DynamicImage,
    file_path: Option<PathBuf>,
    // width: u32,
    // height: u32,
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
) -> Result<DynamicImage, ImageError> {
    let width = canvas.width();
    let height = canvas.height();
    // let data: ImageData = ctx.get_image_data(0.0, 0.0, 100.0, 100.0);
    let pixels = ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .map_err(|err: JsValue| {
            ImageError::IoError(IOError::new(
                ErrorKind::Other,
                err.as_string()
                    .unwrap_or(String::from("failed to read image data from canvas.")),
            ))
        })?
        .data()
        .to_vec();
    // let pixels = data
    // let _len_vec = photon_image.raw_pixels.len() as u128;
    // let raw_pixels = &photon_image.raw_pixels;
    let img_buffer = ImageBuffer::from_vec(width, height, pixels).ok_or_else(|| {
        ImageError::Decoding(DecodingError::new(
            ImageFormatHint::Unknown,
            IOError::new(ErrorKind::Other, "nooo"),
        ))
    })?;
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
    ) -> Result<FilmImage, ImageError> {
        let img = get_image_data(&canvas, &ctx)?;
        Ok(FilmImage {
            img,
            file_path: None,
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
        let img = ImageReader::open(image_path.to_owned())?.decode()?;
        Ok(FilmImage {
            img: img.clone(),
            file_path: Some(image_path),
        })
        // Ok(ImageBorders {
        //     image_path: image_path,
        //     output_path: default_output,
        //     img: img.clone(),
        // })
    }

    pub fn save_to_file(
        &self,
        output_path: Option<String>,
        quality: Option<u8>,
    ) -> Result<(), ImageError> {
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
                // .ok()
                // .ok_or(ImageError::IoError(IOError::new(ErrorKind::Other, "nooo")))?
                .and_then(|name| name.file_stem())
                .and_then(|name| name.to_str())
            {
                Some(stem) => Some(b.join(format!("{}_with_border.jpg", stem))),
                None => None,
            }
        });

        // let filename = // .ok_or_else(|| ImageError::IoError(IOError::new(ErrorKind::Other, "nooo")))? // .ok() // .to_str()
        // .ok() // .ok_or_else(|| ImageError::IoError(IOError::new(
        //     ErrorKind::Other,
        //     "nooo"
        // )))
        // ));
        // Some(filename)
        // });
        let default_output = default_output.and_then(|p| p.into_os_string().into_string().ok());
        let output_path = output_path
            .or(default_output)
            .ok_or(ImageError::IoError(IOError::new(ErrorKind::Other, "nooo")))?;
        println!("saving to {}...", output_path);
        let mut file = File::create(&output_path)?;
        let mut encoder = JpegEncoder::new_with_quality(&mut file, quality.unwrap_or(80));
        encoder.encode_image(&self.img);
        // self.img.save(&output_path.unwrap_or(default_output))?;
        Ok(())
    }
}
