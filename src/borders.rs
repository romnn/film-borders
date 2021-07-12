use crate::img::{Direction, FilmImage};
use crate::utils;
use image::error::ImageError;
use image::imageops::{crop, overlay, resize, rotate180, rotate270, rotate90, FilterType};
use image::{ImageFormat, Pixel, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::fmt;
use std::io::{Error as IOError, ErrorKind};
use std::path::PathBuf;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

static FILM_BORDER_BYTES: &[u8; 170143] = include_bytes!("border.png");

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct OutputSize {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[wasm_bindgen]
impl OutputSize {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        OutputSize::default()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[wasm_bindgen]
impl Size {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Size::default()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Point::default()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct Crop {
    pub top: Option<u32>,
    pub right: Option<u32>,
    pub bottom: Option<u32>,
    pub left: Option<u32>,
}

#[wasm_bindgen]
impl Crop {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Crop::default()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct Sides {
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
}

#[wasm_bindgen]
impl Sides {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Sides::default()
    }
    pub fn uniform(side: u32) -> Sides {
        Sides {
            top: side,
            left: side,
            right: side,
            bottom: side,
        }
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

#[derive(Debug, Clone)]
pub struct ParseEnumError {
    msg: String,
}

impl ParseEnumError {
    pub fn new(msg: String) -> Self {
        ParseEnumError { msg }
    }
}

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::str::FromStr for Rotation {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ss = &*s.to_lowercase().to_owned();
        match ss {
            "270" => Ok(Rotation::Rotate270),
            "180" => Ok(Rotation::Rotate180),
            "90" => Ok(Rotation::Rotate90),
            "0" => Ok(Rotation::Rotate0),
            _ => Err(ParseEnumError::new(format!("unknown rotation: {}", ss))),
        }
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct ImageBorderOptions {
    pub output_size: Option<OutputSize>,
    pub scale_factor: Option<f32>,
    pub crop: Option<Crop>,
    pub border_width: Option<Sides>,
    pub rotate_angle: Option<Rotation>,
    pub preview: bool,
}

#[wasm_bindgen]
impl ImageBorderOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        ImageBorderOptions::default()
    }

    pub fn deserialize(val: String) -> Result<ImageBorderOptions, JsValue> {
        Ok(serde_json::from_str(&val).map_err(|err| JsValue::from_str(&err.to_string()))?)
    }

    pub fn serialize(&self) -> Result<String, JsValue> {
        Ok(serde_json::to_string(&self).map_err(|err| JsValue::from_str(&err.to_string()))?)
    }
}

pub struct ImageBorders {
    img: FilmImage,
}

impl ImageBorders {
    pub fn new(img: FilmImage) -> ImageBorders {
        utils::set_panic_hook();
        ImageBorders { img }
    }

    #[allow(dead_code)]
    pub fn save_jpeg(
        &self,
        buffer: RgbaImage,
        output_path: Option<String>,
        quality: Option<u8>,
    ) -> Result<(), ImageError> {
        self.img.save_jpeg_to_file(buffer, output_path, quality)
    }

    #[allow(dead_code)]
    pub fn save(&self, buffer: RgbaImage, output_path: Option<String>) -> Result<(), ImageError> {
        self.img.save_to_file(buffer, output_path)
    }

    #[allow(dead_code)]
    pub fn store(
        &self,
        img: &RgbaImage,
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<(), JsValue> {
        // Convert the raw pixels back to an ImageData object.
        let img_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(img.as_raw()),
            canvas.width(),
            canvas.height(),
        )?;

        // Place the new imagedata onto the canvas
        ctx.put_image_data(&img_data, 0.0, 0.0)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn apply(&mut self, options: ImageBorderOptions) -> Result<RgbaImage, ImageError> {
        let mut size = Size {
            width: self.img.buffer.width(),
            height: self.img.buffer.height(),
        };
        if let Some(output_size) = options.output_size {
            if let Some(output_width) = output_size.width {
                size.width = output_width;
            };
            if let Some(output_height) = output_size.height {
                size.height = output_height;
            };
        };

        let mut final_image = RgbaImage::new(size.width, size.height);
        let mut photo = self.img.buffer.clone();
        let output_is_portrait = size.width <= size.height;
        let rem = max(size.width, size.height) as f32 / 1000.0;

        // fill background
        let bg_color = if options.preview {
            Rgba([200, 200, 200, 255])
        } else {
            Rgba([255, 255, 255, 255])
        };
        FilmImage::fill_rect(
            &mut final_image,
            bg_color,
            Point { x: 0, y: 0 },
            Point {
                x: size.width,
                y: size.height,
            },
        );

        // rotate the image
        if let Some(rotate_angle) = options.rotate_angle {
            photo = match rotate_angle {
                Rotation::Rotate0 => photo,
                Rotation::Rotate90 => rotate90(&photo),
                Rotation::Rotate180 => rotate180(&photo),
                Rotation::Rotate270 => rotate270(&photo),
            };
        };

        let photo_is_portrait = photo.width() <= photo.height();

        // crop the image
        if let Some(crop_opts) = options.crop {
            let crop_top = (crop_opts.top.unwrap_or(0) as f32 * rem) as u32;
            let crop_right = photo.width() - ((crop_opts.right.unwrap_or(0) as f32 * rem) as u32);
            let crop_bottom =
                photo.height() - ((crop_opts.bottom.unwrap_or(0) as f32 * rem) as u32);
            let crop_left = (crop_opts.left.unwrap_or(0) as f32 * rem) as u32;

            let crop_width = max(0, crop_right as i64 - crop_left as i64) as u32;
            let crop_height = max(0, crop_bottom as i64 - crop_top as i64) as u32;
            photo = crop(&mut photo, crop_left, crop_top, crop_width, crop_height).to_image()
        };

        // resize the image to fit the screen
        let (mut fit_width, mut fit_height) = utils::resize_dimensions(
            photo.width(),
            photo.height(),
            size.width,
            size.height,
            false,
        );
        // println!("fitting to {} x {}", fit_width, fit_height);

        if let Some(scale_factor) = options.scale_factor {
            // scale the image by factor
            fit_width = (fit_width as f32 * utils::clamp(scale_factor, 0f32, 1f32)) as u32;
            fit_height = (fit_height as f32 * utils::clamp(scale_factor, 0f32, 1f32)) as u32;
            // println!("scaling to {} x {}", fit_width, fit_height);
        };

        photo = resize(&photo, fit_width, fit_height, FilterType::Lanczos3);

        let overlay_x = (size.width - photo.width()) / 2;
        let overlay_y = (size.height - photo.height()) / 2;
        // println!("overlaying at {} {}", overlay_x, overlay_y);

        // create the black borders
        if let Some(border_width) = options.border_width {
            let black_color = Rgba::from_channels(0, 0, 0, 255);
            let top_left = Point {
                x: max(
                    0,
                    overlay_x as i32 - (border_width.left as f32 * rem) as i32,
                ) as u32,
                y: max(0, overlay_y as i32 - (border_width.top as f32 * rem) as i32) as u32,
            };
            let btm_right = Point {
                x: max(
                    0,
                    (overlay_x + photo.width()) as i32 + (border_width.right as f32 * rem) as i32,
                ) as u32,
                y: max(
                    0,
                    (overlay_y + photo.height()) as i32 + (border_width.bottom as f32 * rem) as i32,
                ) as u32,
            };
            FilmImage::fill_rect(&mut final_image, black_color, top_left, btm_right);
        };

        overlay(&mut final_image, &photo, overlay_x, overlay_y);

        // add the film borders
        let mut fb = image::load_from_memory_with_format(FILM_BORDER_BYTES, ImageFormat::Png)?
            .as_rgba8()
            .ok_or(ImageError::IoError(IOError::new(
                ErrorKind::Other,
                "failed to read film border image data",
            )))?
            .clone();

        if photo_is_portrait {
            fb = rotate90(&fb);
        };
        let mut fb_width = fit_width;
        let mut fb_height = (fb.height() as f32 * (fit_width as f32 / fb.width() as f32)) as u32;
        if !photo_is_portrait {
            fb_height = fit_height;
            fb_width = (fb.width() as f32 * (fit_height as f32 / fb.height() as f32)) as u32;
        };
        fb = resize(&fb, fb_width, fb_height, FilterType::Lanczos3);

        let fade_transition_direction = if photo_is_portrait {
            Direction::Vertical
        } else {
            Direction::Horizontal
        };
        let fade_width = (0.05 * fit_height as f32) as u32;
        let fb_useable_frac = 0.75;

        // top border
        let mut top_fb = fb.clone();
        let top_fb_crop = Size {
            width: if photo_is_portrait {
                fb.width()
            } else {
                min(
                    (fb_useable_frac * photo.width() as f32) as u32,
                    (fb_useable_frac * fb.width() as f32) as u32,
                )
            },
            height: if photo_is_portrait {
                min(
                    (fb_useable_frac * photo.height() as f32) as u32,
                    (fb_useable_frac * fb.height() as f32) as u32,
                )
            } else {
                fb.height()
            },
        };
        top_fb = crop(&mut top_fb, 0, 0, top_fb_crop.width, top_fb_crop.height).to_image();
        let fade_dim = if photo_is_portrait {
            top_fb_crop.height
        } else {
            top_fb_crop.width
        };
        FilmImage::fade_out(
            &mut top_fb,
            max(0, fade_dim - fade_width),
            fade_dim - 1,
            fade_transition_direction,
        );
        overlay(&mut final_image, &top_fb, overlay_x, overlay_y);

        // bottom border
        let mut btm_fb = fb.clone();
        let btm_fb_crop = Size {
            width: if photo_is_portrait {
                fb.width()
            } else {
                min(
                    (fb_useable_frac * photo.width() as f32) as u32,
                    (fb_useable_frac * fb.width() as f32) as u32,
                )
            },
            height: if photo_is_portrait {
                min(
                    (fb_useable_frac * photo.height() as f32) as u32,
                    (fb_useable_frac * fb.height() as f32) as u32,
                )
            } else {
                fb.height()
            },
        };
        let btm_fb_x = if photo_is_portrait {
            0
        } else {
            btm_fb.width() - btm_fb_crop.width
        };
        let btm_fb_y = if photo_is_portrait {
            btm_fb.height() - btm_fb_crop.height
        } else {
            0
        };

        btm_fb = crop(
            &mut btm_fb,
            btm_fb_x,
            btm_fb_y,
            btm_fb_crop.width,
            btm_fb_crop.height,
        )
        .to_image();
        FilmImage::fade_out(&mut btm_fb, fade_width, 0, fade_transition_direction);
        overlay(
            &mut final_image,
            &btm_fb,
            if photo_is_portrait {
                overlay_x
            } else {
                overlay_x + (photo.width() - btm_fb_crop.width)
            },
            overlay_y + (fit_height - btm_fb_crop.height),
        );

        // intermediate borders
        let inter_fb_crop = Size {
            width: if photo_is_portrait {
                fb.width()
            } else {
                min(
                    (0.5 * photo.width() as f32) as u32,
                    (0.5 * fb.width() as f32) as u32,
                )
            },
            height: if photo_is_portrait {
                min(
                    (0.5 * photo.height() as f32) as u32,
                    (0.5 * fb.height() as f32) as u32,
                )
            } else {
                fb.height()
            },
        };

        let (start, end, step_size) = if photo_is_portrait {
            (
                top_fb_crop.height - fade_width,
                fit_height - btm_fb_crop.height + fade_width,
                inter_fb_crop.height as usize,
            )
        } else {
            (
                top_fb_crop.width - fade_width,
                fit_width - btm_fb_crop.width + fade_width,
                inter_fb_crop.width as usize,
            )
        };

        // println!("from {} to {} with step size {}", start, end, step_size);
        for i in (start..=end).step_by(step_size) {
            let mut inter_fb = fb.clone();
            let (inter_fb_x, inter_fb_y, inter_fb_width, inter_fb_height) = if photo_is_portrait {
                (
                    0,
                    (0.25 * fb.height() as f32) as u32 - fade_width,
                    inter_fb_crop.width,
                    min(inter_fb_crop.height, end - i) + 2 * fade_width,
                )
            } else {
                (
                    (0.25 * fb.width() as f32) as u32 - fade_width,
                    0,
                    min(inter_fb_crop.width, end - i) + 2 * fade_width,
                    inter_fb_crop.height,
                )
            };
            inter_fb = crop(
                &mut inter_fb,
                inter_fb_x,
                inter_fb_y,
                inter_fb_width,
                inter_fb_height,
            )
            .to_image();
            FilmImage::fade_out(&mut inter_fb, fade_width, 0, fade_transition_direction);
            let fade_dim = if photo_is_portrait {
                inter_fb_height
            } else {
                inter_fb_width
            };
            FilmImage::fade_out(
                &mut inter_fb,
                fade_dim - fade_width,
                fade_dim - 1,
                fade_transition_direction,
            );
            overlay(
                &mut final_image,
                &inter_fb,
                if photo_is_portrait {
                    overlay_x
                } else {
                    overlay_x - fade_width + i
                },
                if photo_is_portrait {
                    overlay_y - fade_width + i
                } else {
                    overlay_y
                },
            );
        }

        // show the center of the final image
        if options.preview {
            let highlight_color = Rgba::from_channels(255, 0, 0, 50);
            let mut ctr_tl = Point {
                x: 0,
                y: (size.height - size.width) / 2,
            };
            let mut ctr_br = Point {
                x: size.width,
                y: ((size.height - size.width) / 2) + size.width,
            };
            if !output_is_portrait {
                ctr_tl = Point {
                    x: (size.width - size.height) / 2,
                    y: 0,
                };
                ctr_br = Point {
                    x: ((size.width - size.height) / 2) + size.height,
                    y: size.height,
                };
            }
            FilmImage::fill_rect(&mut final_image, highlight_color, ctr_tl, ctr_br);
        };
        Ok(final_image)
    }

    #[allow(dead_code)]
    pub fn from_file(input_path: PathBuf) -> Result<ImageBorders, ImageError> {
        let img = FilmImage::from_file(input_path)?;
        Ok(ImageBorders { img })
    }
}
