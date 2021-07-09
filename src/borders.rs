use crate::img::{Direction, FilmImage};
use crate::utils;
use image::error::{DecodingError, ImageError, ImageFormatHint, ImageResult};
use image::imageops::{crop, overlay, resize, rotate180, rotate270, rotate90, FilterType};
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer, ImageFormat, Pixel, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::env;
use std::error::Error;
use std::io::Cursor;
use std::io::{Error as IOError, ErrorKind};
use std::path::{Path, PathBuf};
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

static film_border_bytes: &[u8; 170143] = include_bytes!("border.png");

#[wasm_bindgen]
#[derive(Default, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[wasm_bindgen]
impl Size {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Size {
        Size::default()
    }
}

#[wasm_bindgen]
#[derive(Debug, Default, Copy, Clone)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Point {
        Point::default()
    }
}

#[wasm_bindgen]
#[derive(Debug, Default, Copy, Clone)]
pub struct Crop {
    pub top_left: Point,
    pub bottom_right: Point,
}

#[wasm_bindgen]
impl Crop {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Crop {
        Crop::default()
    }
}

#[wasm_bindgen]
#[derive(Debug, Default, Copy, Clone)]
pub struct Sides {
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
}

#[wasm_bindgen]
impl Sides {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Sides {
        Sides::default()
    }
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

#[wasm_bindgen]
#[derive(Default, Copy, Clone)]
pub struct ImageBorderOptions {
    pub output_size: Option<Size>,
    pub scale_factor: Option<f32>,
    pub crop: Option<Crop>,
    pub border_width: Option<Sides>,
    pub rotate_angle: Option<Rotation>,
    pub preview: bool,
}

#[wasm_bindgen]
impl ImageBorderOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ImageBorderOptions {
        ImageBorderOptions::default()
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

    pub fn store(
        &self,
        img: &RgbaImage,
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<(), JsValue> {
        // Convert the raw pixels back to an ImageData object.
        let img_data = ImageData::new_with_u8_clamped_array_and_sh(
            // Clamped(&img.raw_pixels),
            Clamped(img.as_raw()),
            canvas.width(),
            canvas.height(),
        )?;

        // Place the new imagedata onto the canvas
        ctx.put_image_data(&img_data, 0.0, 0.0)?;
        Ok(())
        // .expect("Should put image data on Canvas");
        // Ok(())
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
        if false && options.preview {
            // deprecated: changing the size of result does not speed things up much
            size = Size {
                width: (size.width as f32 * 0.25) as u32,
                height: (size.height as f32 * 0.25) as u32,
            };
        };

        let mut final_image = RgbaImage::new(size.width, size.height);
        let mut photo = self.img.buffer.clone();
        let output_is_portrait = size.width <= size.height;
        let rem = size.width as f32 / 1000.0;

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
            photo = crop(&mut photo, crop_x, crop_y, crop_width, crop_height).to_image()
        };

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

        // resize the image to fit the screen
        let (mut fit_width, mut fit_height) = utils::resize_dimensions(
            photo.width(),
            photo.height(),
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

        let fitted_image = resize(&photo, fit_width, fit_height, FilterType::Lanczos3);

        let overlay_x = (size.width - fitted_image.width()) / 2;
        let overlay_y = (size.height - fitted_image.height()) / 2;
        println!("overlaying at {} {}", overlay_x, overlay_y);

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
                    (overlay_x + fitted_image.width()) as i32
                        + (border_width.right as f32 * rem) as i32,
                ) as u32,
                y: max(
                    0,
                    (overlay_y + fitted_image.height()) as i32
                        + (border_width.bottom as f32 * rem) as i32,
                ) as u32,
            };
            // println!("filling from {:?} to {:?}", top_left, btm_right);
            FilmImage::fill_rect(&mut final_image, black_color, top_left, btm_right);
        };

        overlay(&mut final_image, &fitted_image, overlay_x, overlay_y);

        // add the film borders
        let mut film_borders =
            image::load_from_memory_with_format(film_border_bytes, ImageFormat::Png)?
                .as_rgba8()
                .ok_or(ImageError::IoError(IOError::new(
                    ErrorKind::Other,
                    "failed to read film border image data",
                )))?
                .clone();
        println!("is portrait: {}", photo_is_portrait);
        if photo_is_portrait {
            film_borders = rotate90(&film_borders);
        };
        film_borders = resize(
            &film_borders,
            fit_width,
            (film_borders.height() as f32 * (fit_width as f32 / film_borders.width() as f32))
                as u32,
            FilterType::Lanczos3,
        );

        let fade_transition_direction = if photo_is_portrait {
            Direction::Vertical
        } else {
            Direction::Horizontal
        };
        // println!("direction: {:?}", fade_transition_direction);
        let fade_width = (0.05 * fit_height as f32) as u32;
        let fb_useable_frac = 0.2;

        // top border
        let mut top_fb = film_borders.clone();
        let top_fb_crop = Size {
            width: film_borders.width(),
            height: min(
                (fb_useable_frac * photo.height() as f32) as u32,
                (fb_useable_frac * film_borders.height() as f32) as u32,
            ),
        };
        top_fb = crop(&mut top_fb, 0, 0, top_fb_crop.width, top_fb_crop.height).to_image();
        FilmImage::fade_out(
            &mut top_fb,
            max(0, top_fb_crop.height - fade_width),
            top_fb_crop.height - 1,
            fade_transition_direction,
        );
        overlay(&mut final_image, &top_fb, overlay_x, overlay_y);

        // bottom border
        let mut btm_fb = film_borders.clone();
        let btm_fb_crop = Size {
            width: film_borders.width(),
            height: min(
                (fb_useable_frac * photo.height() as f32) as u32,
                (fb_useable_frac * film_borders.height() as f32) as u32,
            ),
        };
        let btm_fb_y = btm_fb.height() - btm_fb_crop.height;
        btm_fb = crop(
            &mut btm_fb,
            0,
            btm_fb_y,
            btm_fb_crop.width,
            btm_fb_crop.height,
        )
        .to_image();
        FilmImage::fade_out(&mut btm_fb, fade_width, 0, fade_transition_direction);
        overlay(
            &mut final_image,
            &btm_fb,
            overlay_x,
            overlay_y + (fit_height - btm_fb_crop.height),
        );

        // intermediate borders
        let inter_fb_crop = Size {
            width: film_borders.width(),
            height: min(
                (0.5 * photo.height() as f32) as u32,
                (0.5 * film_borders.height() as f32) as u32,
            ),
        };

        println!("step size is {}", inter_fb_crop.height);
        let end = fit_height - btm_fb_crop.height;
        println!("from {} to {}", top_fb_crop.height - fade_width, end);

        for i in (top_fb_crop.height..=end).step_by(inter_fb_crop.height as usize) {
            println!("{}", i);
            let mut inter_fb = film_borders.clone();
            let inter_fb_height = min(inter_fb_crop.height, end - i);
            let inter_fb_crop_y = (0.25 * film_borders.height() as f32) as u32;
            println!("crop y is {}", inter_fb_crop_y);
            inter_fb = crop(
                &mut inter_fb,
                0,
                inter_fb_crop_y - fade_width,
                inter_fb_crop.width,
                inter_fb_height + 2 * fade_width,
            )
            .to_image();
            FilmImage::fade_out(&mut inter_fb, fade_width, 0, fade_transition_direction);
            FilmImage::fade_out(
                &mut inter_fb,
                inter_fb_height + fade_width,
                inter_fb_height + 2 * fade_width - 1,
                fade_transition_direction,
            );
            overlay(
                &mut final_image,
                &inter_fb,
                overlay_x,
                overlay_y - fade_width + i,
            );
        }

        // show the center of the final image
        if options.preview {
            let highlight_color = Rgba::from_channels(255, 0, 0, 50);
            FilmImage::fill_rect(
                &mut final_image,
                highlight_color,
                Point {
                    x: 0,
                    y: (size.height - size.width) / 2,
                },
                Point {
                    x: size.width,
                    y: ((size.height - size.width) / 2) + size.width,
                },
            );
        };
        // let line_width = 10;
        // let top_line = (Point{ x: 0, y: (size.height - size.width) / 2

        Ok(final_image)
    }

    pub fn from_file(input_path: PathBuf) -> Result<ImageBorders, ImageError> {
        let img = FilmImage::from_file(input_path)?;
        Ok(ImageBorders { img })
    }
}
