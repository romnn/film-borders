#[cfg(feature = "borders")]
pub mod borders;
pub mod debug;
pub mod defaults;
pub mod error;
pub mod imageops;
pub mod img;
pub mod options;
pub mod types;
pub mod utils;
#[cfg(feature = "wasm")]
pub mod wasm;

pub use error::Error;
pub use image::ImageFormat;
pub use img::Image;
pub use options::*;
pub use types::*;

use approx;
#[cfg(debug_assertions)]
use chrono::Utc;
use image::{DynamicImage, Rgba, RgbaImage};
use std::cmp::{max, min};
use std::path::Path;

pub struct ImageBorders {
    images: Vec<img::Image>,
}

impl ImageBorders {
    pub fn new(images: Vec<img::Image>) -> Result<ImageBorders, Error> {
        if images.is_empty() {
            Err(Error::MissingImage)
        } else {
            Ok(ImageBorders { images })
        }
    }

    pub fn single(img: img::Image) -> ImageBorders {
        ImageBorders { images: vec![img] }
    }

    pub fn from_reader<R: std::io::BufRead + std::io::Seek>(reader: R) -> Result<Self, Error> {
        let img = Image::from_reader(reader)?;
        Ok(Self::single(img))
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let img = Image::open(path)?;
        Ok(Self::single(img))
    }

    pub fn add_border(
        &mut self,
        border: Option<BorderSource>,
        options: &Options,
    ) -> Result<img::Image, Error> {
        // prepare images
        let mut images: Vec<img::Image> = self.images.clone();
        let primary = images.get_mut(0).ok_or(Error::MissingImage)?;
        if let Some(angle) = options.rotate_angle {
            primary.rotate(angle);
        }
        if let Some(crop_percent) = options.crop {
            let crop = crop_percent * primary.size();
            primary.crop_sides(crop);
        };

        // border.map(BorderSource::into_border); // ()?;

        // prepare the border for the primary image
        let mut border = match border {
            Some(border) => {
                let mut border = border.into_border()?;
                border.rotate_to_orientation(primary.orientation())?;
                Some(border)
            }
            None => None,
        };

        // let default_padding = 3;
        let original_content_size = match border {
            Some(ref mut border) => match options.mode {
                Mode::FitImage => {
                    let border_size = border.size_for(primary.size());
                    border_size
                }
                Mode::FitBorder => {
                    // create a new custom border
                    let mut test = Border::new(border.clone(), primary.size(), None)?;
                    *border = test;
                    // border = test;
                    border.size()
                    // let padding = min(default_padding, p
                    // let border_padding = border.size() - comp.size();
                    // primary.size() + border_padding
                }
            },
            None => primary.size(),
        };
        crate::debug!("image with border size: {}", &original_content_size);
        // .min(options.output_size_bounds);
        // .scale_to(options.output_size_bounds, types::ResizeMode::Contain);
        // crate::debug!(output_size - content_size);

        // min(output_size.width, output_size.height);
        // let base = min(output_size.width, output_size.height);
        // let frame_width = 10; // options.frame_width * base;
        // let margin = 20; // ((1.0 - scale_factor) * base as f32) as u32;
        // crate::debug!(&options.output_size);

        let scale_factor = utils::clamp(options.scale_factor, 0.0, 1.0);
        let margin_factor = options.margin.max(0.0);

        let base = original_content_size.min_dim();
        let frame_width: Sides = options.frame_width * base;
        let margin: Sides = Sides::uniform((margin_factor * base as f32) as u32);
        let mut content_size = original_content_size + frame_width + margin;
        // content_size = content_size * scale_factor;

        let default_output_size = content_size * (1.0 / scale_factor);

        // set output size and do not keep aspect ratio
        let output_size =
            default_output_size.scale_to_bounds(options.output_size, types::ResizeMode::Fill);
        let output_size =
            output_size.scale_to_bounds(options.output_size_bounds, types::ResizeMode::Contain);
        // match options.output_size {
        // }

        // set output size and keep aspect ratio
        // let output_size = match options.output_size.min(options.output_size_bounds) {
        // let output_size = default_output_size;

        // let target_output_size = original_content_size + frame_width + margin;
        // let output_size = .scale_to(options.output_size);

        // let scale_factor = output_size / target_output_size;
        // let frame_width = frame_width * scale_factor;
        // let margin = margin * scale_factor;

        // create new result image
        let mut result_image = img::Image::with_size(output_size);
        result_image.path = primary.path.clone();

        let background_color = options.background_color.unwrap_or(if options.preview {
            Color::gray()
        } else {
            Color::white()
        });
        result_image.fill(background_color);

        // content size aspect ratio is equal to border one

        // crate::debug!(border.size() + frame_width + margin);
        // crate::debug!(border.size());
        // crate::debug!(frame_width + margin);

        // assert_eq!(content_size - border.size(), (frame_width + margin).into());
        // assert_eq!(
        //     (border.size() + frame_width + margin).aspect_ratio(),
        //     border.size().aspect_ratio()
        // );

        // approx::abs_diff_eq!(
        //     content_size.aspect_ratio(),
        //     original_content_size.aspect_ratio(),
        //     epsilon = 0.01
        // );

        // assert_eq!(content_size.aspect_ratio(), border.size().aspect_ratio());

        // let temp_content_rect = output_size.center(content_size);

        // let base = min(content_size.width, content_size.height);
        // let new_base = output_size.min_dim() / base;
        // let content_size = original_content_size.scale_to(output_size, types::ResizeMode::Contain);
        // let content_size = content_size + frame_width + margin;

        let new_content_size = content_size.scale_to(
            output_size * scale_factor, //  - frame_width - margin,
            types::ResizeMode::Contain,
        );
        // let scale = new_content_size.min_dim() / content_size.min_dim();
        let scale = new_content_size.min_dim() as f64 / content_size.min_dim() as f64;
        crate::debug!(&scale);
        let frame_width = frame_width * scale;
        crate::debug!(&frame_width);
        let margin = margin * scale;
        crate::debug!(&margin);

        // let content_size = content_size.scale_to(
        //     output_size - frame_width - margin,
        //     types::ResizeMode::Contain,
        // );

        // let content_size = output_size - frame_width - margin;
        // let content_size = content_size + frame_width + margin;

        // let content_size = content_size - margin;
        // let content_size = content_size.scale_to(output_size - margin, types::ResizeMode::Contain);

        // content_size.scale_to(output_size * scale_factor, types::ResizeMode::Contain);
        let content_rect = output_size.center(new_content_size);

        #[cfg(debug_assertions)]
        imageops::fill_rect(
            &mut result_image,
            types::Color::rgba(0, 0, 255, 100),
            content_rect.top_left(),
            content_rect.size(),
        );

        //
        // let content_rect = content_rect - margin;
        //
        // crate::debug!(output_size - content_size);
        // crate::debug!(&content_size);
        crate::debug!(&content_rect);

        imageops::fill_rect(
            &mut result_image,
            options.frame_color,
            (content_rect - margin).top_left(),
            (content_rect - margin).size(),
        );

        // create the border frame
        // todo: use a rect for stuff like this!!
        // let frame_top_left = (Point::from(output_size) - Point::from(content_size)) / 2.0f64;
        // // frame_top_left = frame_top_left / 2.0f64;
        // crate::debug!(&frame_top_left);

        // imageops::fill_rect(
        //     &mut result_image,
        //     options.frame_color,
        //     content_rect.top_left(),
        //     content_rect.size(),
        // );

        // return Ok(result_image);
        // todo!();

        let border_rect = content_rect - margin - frame_width;

        crate::debug!("overlay content now");
        match options.mode {
            Mode::FitImage => {
                // let frame_width = 10;
                // let border_rect = content_rect - Sides::uniform(frame_width); // .into();
                // Sides::uniform(frame_width); // .into();
                crate::debug!(&content_rect);
                crate::debug!(&frame_width);
                crate::debug!(&border_rect);
                // let border_size = content_size - frame_width;
                // let border_top_left =
                //     (Point::from(output_size) - Point::from(border_size)) / 2.0f32;
                // crate::debug!(&border_size);
                // crate::debug!(&border.size());
                // crate::debug!(&border_top_left);
                // border.resize_to_fit(border_size, types::ResizeMode::Contain)?;
                // if let Some(ref mut border) = border {
                //     border.resize_to_fit(border_rect.size(), types::ResizeMode::Contain)?;
                // }

                let default_component = vec![border_rect];
                let mut components = match border {
                    Some(ref mut border) => {
                        border.resize_to_fit(border_rect.size(), types::ResizeMode::Contain)?;

                        let default_image = primary.clone();
                        images.resize(border.transparent_components().len(), default_image);
                        border
                            .transparent_components()
                            // .to_vec()
                            // .into_iter()
                            .iter()
                            .zip(images.iter_mut())
                    }
                    None => default_component.iter().zip(images.iter_mut()),
                };

                for (c, image) in components {
                    crate::debug!("drawing {:?}", &c);
                    // crate::debug!(&c.size());
                    // crate::debug!(&c.top_left());
                    let mut image_rect = *c + border_rect.top_left();
                    image_rect = image_rect.extend(3);
                    image_rect = image_rect.clip_to(&border_rect);

                    let crop_mode = image_rect.crop_mode(&border_rect);
                    // image_rect.center()
                    image.resize_to_fit(
                        image_rect.size(),
                        types::ResizeMode::Cover,
                        // types::CropMode::Center,
                        crop_mode,
                        // types::CropMode::Right,
                    );

                    // let c_top_left = border_top_left + c.top_left();
                    // let top_left: Point = c_top_left - padding.top_left();
                    // must limit to content size
                    // let top_left = border_top_left + image_rect.top_left();
                    result_image.overlay(image.as_ref(), image_rect.top_left());
                }

                // THIS IS ONLY VALID FOR ZERO FRAME
                // crate::debug!(border_rect.top_left() == content_rect.top_left());
                // crate::debug!(border.size() == content_rect.size());
                if let Some(border) = border {
                    result_image.overlay(border.as_ref(), border_rect.top_left());
                }
            }
            Mode::FitBorder => {
                // let (c, image) = components.next().ok_or(Error::MissingImage)?;
                // let primary_size = border.size_for(primary.size());
                let c = match border {
                    Some(ref mut border) => {
                        border.resize_to_fit(border_rect.size(), types::ResizeMode::Contain)?;
                        border.content_rect()
                    }
                    None => &border_rect,
                };

                let mut image_rect = *c + border_rect.top_left();
                image_rect = image_rect.extend(3);
                image_rect = image_rect.clip_to(&border_rect);

                primary.resize_to_fit(
                    image_rect.size(),
                    types::ResizeMode::Cover,
                    types::CropMode::Center,
                    // crop_mode,
                    // types::CropMode::Right,
                );

                result_image.overlay(primary.as_ref(), image_rect.top_left());
                if let Some(border) = border {
                    result_image.overlay(border.as_ref(), border_rect.top_left());
                }
            }
        };

        // compute default content size
        // let border_size = border.size_for(primary.size());
        // crate::debug!(&border_size);
        // crate::debug!(&border.transparent_components());

        // let mut primary = self.images.first().ok_or(Error::MissingImage)?;
        // .clone();
        // crate::debug!(&primary.size());

        // let mode = options.mode.unwrap_or(Default::default());
        // let scale_factor = options.scale_factor.unwrap_or(1.0);
        // let rem = primary.size().max() as f32 / 1000.0;
        // let rem = 1.0f32;

        // types::OutputSize::default();
        // let output_size = Size {
        //     width: options
        //         .output_size
        //         .and_then(|size| size.width)
        //         .unwrap_or(default_output_size.width),
        //     height: options
        //         .output_size
        //         .and_then(|size| size.height)
        //         .unwrap_or(default_output_size.height),
        // };
        // crate::debug!(&output_size);
        // scale to absolute limits
        //

        // scale the default content size into a frame of the old content_size
        //     or the respective smaller bound given by max size

        // let content_size = utils::resize_dimensions(
        //     content_size,
        //     output_size - frame_width
        //     utils::ResizeMode::Contain,
        // }

        // // resize the image to fit the screen
        // let (mut fit_width, mut fit_height) = utils::resize_dimensions(
        //     primary.size(),
        //     output_size.size(),
        //     // photo.width(),
        //     // photo.height(),
        //     // size.width,
        //     // size.height,
        //     // false,
        // );

        // if let Some(scale_factor) = options
        //     .scale_factor
        //     .map(|scale| utils::clamp(scale, 0.05f32, 1f32))
        // {
        //     // scale the image by factor
        //     fit_width = (fit_width as f32 * scale_factor) as u32;
        //     fit_height = (fit_height as f32 * scale_factor) as u32;
        //     crate::debug!("scaling to {} x {}", fit_width, fit_height);
        // };

        // match
        // let images: Vec<(Component, img::Image)> =
        // for (component, img) in             // .cloned()
        // .map(|(component, image)| {
        //     (component, image)
        // })
        // .collect();
        // {}

        // let (_, mut primary) = images.first().ok_or(Error::MissingImage)?;
        // let mut primary = &mut images[0]; // .first().ok_or(Error::MissingImage)?;

        // #[cfg(debug_assertions)]
        // let start = Utc::now().time();
        // primary.resize_to_fit()
        // photo = imageops::resize(&photo, fit_width, fit_height, defaults::FILTER_TYPE);
        // crate::debug!(
        //     "fitting to {} x {} took {:?}",
        //     fit_width,
        //     fit_height,
        //     Utc::now().time() - start,
        // );

        // let overlay_x = ((size.width - photo.width()) / 2) as i64;
        // let overlay_y = ((size.height - photo.height()) / 2) as i64;
        // crate::debug!("overlaying at {} {}", overlay_x, overlay_y);

        // prepare the image
        // size is either: unmodified, or scale to fit size

        // // this creates a copy
        // let mut photo = self.img.data();
        // let result_is_portrait = size.width <= size.height;
        // let rem = max(size.width, size.height) as f32 / 1000.0;

        // // rotate the image
        // if let Some(rotate_angle) = options.rotate_angle {
        //     photo = match rotate_angle {
        //         Rotation::Rotate0 => photo,
        //         Rotation::Rotate90 => imageops::rotate90(&photo),
        //         Rotation::Rotate180 => imageops::rotate180(&photo),
        //         Rotation::Rotate270 => imageops::rotate270(&photo),
        //     };
        // };

        // let photo_is_portrait = photo.width() <= photo.height();

        // // crop the image
        // if let Some(crop_opts) = options.crop {
        //     let crop_top = (crop_opts.top as f32 * rem) as u32;
        //     let crop_bottom = (crop_opts.bottom as f32 * rem) as u32;
        //     let crop_right = (crop_opts.right as f32 * rem) as u32;
        //     let crop_left = (crop_opts.left as f32 * rem) as u32;

        //     // let crop_top =
        //     let crop_right = photo.width() - crop_right;
        //     // ((crop_opts.right.unwrap_or(0) as f32 * rem) as u32);
        //     let crop_bottom = photo.height() - crop_bottom;
        //     // photo.height() - ((crop_opts.bottom.unwrap_or(0) as f32 * rem) as u32);
        //     // let crop_left = (crop_opts.left.unwrap_or(0) as f32 * rem) as u32;

        //     let cropped_width = crop_right as i64 - crop_left as i64;
        //     let cropped_width = max(0, cropped_width);
        //     let cropped_height = crop_bottom as i64 - crop_top as i64;
        //     let cropped_height = max(0, cropped_height);

        //     // let cropped_width = max(0, crop_right as i64 - crop_left as i64) as u32;
        //     // let cropped_height = max(0, crop_bottom as i64 - crop_top as i64) as u32;
        //     photo = imageops::crop(
        //         &mut photo,
        //         crop_left,
        //         crop_top,
        //         cropped_width as u32,
        //         cropped_height as u32,
        //     )
        //     .to_image();
        // };

        // // resize the image to fit the screen
        // let (mut fit_width, mut fit_height) = utils::resize_dimensions(
        //     photo.width(),
        //     photo.height(),
        //     size.width,
        //     size.height,
        //     false,
        // );
        // if let Some(scale_factor) = options
        //     .scale_factor
        //     .map(|scale| utils::clamp(scale, 0.05f32, 1f32))
        // {
        //     // scale the image by factor
        //     fit_width = (fit_width as f32 * scale_factor) as u32;
        //     fit_height = (fit_height as f32 * scale_factor) as u32;
        //     crate::debug!("scaling to {} x {}", fit_width, fit_height);
        // };

        // #[cfg(debug_assertions)]
        // let start = Utc::now().time();
        // photo = imageops::resize(&photo, fit_width, fit_height, defaults::FILTER_TYPE);
        // crate::debug!(
        //     "fitting to {} x {} took {:?}",
        //     fit_width,
        //     fit_height,
        //     Utc::now().time() - start,
        // );

        // let overlay_x = ((size.width - photo.width()) / 2) as i64;
        // let overlay_y = ((size.height - photo.height()) / 2) as i64;
        // crate::debug!("overlaying at {} {}", overlay_x, overlay_y);

        // // create the black borders
        // if let Some(border_width) = options.border_width {
        //     let black_color = Rgba([0, 0, 0, 255]);
        //     let top_left = Point {
        //         x: max(
        //             0,
        //             overlay_x as i32 - (border_width.left as f32 * rem) as i32,
        //         ) as u32,
        //         y: max(0, overlay_y as i32 - (border_width.top as f32 * rem) as i32) as u32,
        //     };
        //     let btm_right = Point {
        //         x: max(
        //             0,
        //             (overlay_x + photo.width() as i64) as i32
        //                 + (border_width.right as f32 * rem) as i32,
        //         ) as u32,
        //         y: max(
        //             0,
        //             (overlay_y + photo.height() as i64) as i32
        //                 + (border_width.bottom as f32 * rem) as i32,
        //         ) as u32,
        //     };
        //     imageops::fill_rect(&mut result_image, &black_color, top_left, btm_right);
        // };

        // imageops::overlay(&mut result_image, &photo, overlay_x, overlay_y);

        // if photo_is_portrait {
        //     border = imageops::rotate90(&border);
        // };
        // let mut border_width = fit_width;
        // let mut border_height = (border.height() as f32 * (fit_width as f32 / border.width() as f32)) as u32;
        // if !photo_is_portrait {
        //     border_height = fit_height;
        //     border_width = (border.width() as f32 * (fit_height as f32 / border.height() as f32)) as u32;
        // };
        // #[cfg(debug_assertions)]
        // let start = Utc::now().time();
        // let filter_type = imageops::FilterType::Triangle;
        // border = imageops::resize(&border, border_width, border_height, filter_type);
        // crate::debug!(
        //     "fitting border to {} x {} took {:?}",
        //     fb_width,
        //     fb_height,
        //     Utc::now().time() - start,
        // );

        // let fade_transition_direction = if photo_is_portrait {
        //     imageops::Direction::Vertical
        // } else {
        //     imageops::Direction::Horizontal
        // };
        // let fade_width = (0.05 * fit_height as f32) as u32;
        // let fb_useable_frac = 0.75;

        // // top border
        // let mut top_fb = fb.clone();
        // let top_fb_crop = Size {
        //     width: if photo_is_portrait {
        //         fb.width()
        //     } else {
        //         min(
        //             (fb_useable_frac * photo.width() as f32) as u32,
        //             (fb_useable_frac * fb.width() as f32) as u32,
        //         )
        //     },
        //     height: if photo_is_portrait {
        //         min(
        //             (fb_useable_frac * photo.height() as f32) as u32,
        //             (fb_useable_frac * fb.height() as f32) as u32,
        //         )
        //     } else {
        //         fb.height()
        //     },
        // };
        // top_fb =
        //     imageops::crop(&mut top_fb, 0, 0, top_fb_crop.width, top_fb_crop.height).to_image();
        // let fade_dim = if photo_is_portrait {
        //     top_fb_crop.height
        // } else {
        //     top_fb_crop.width
        // };
        // imageops::fade_out(
        //     &mut top_fb,
        //     max(0, fade_dim - fade_width),
        //     fade_dim - 1,
        //     fade_transition_direction,
        // );
        // imageops::overlay(&mut result_image, &top_fb, overlay_x, overlay_y);

        // // bottom border
        // let mut btm_fb = fb.clone();
        // let btm_fb_crop = Size {
        //     width: if photo_is_portrait {
        //         fb.width()
        //     } else {
        //         min(
        //             (fb_useable_frac * photo.width() as f32) as u32,
        //             (fb_useable_frac * fb.width() as f32) as u32,
        //         )
        //     },
        //     height: if photo_is_portrait {
        //         min(
        //             (fb_useable_frac * photo.height() as f32) as u32,
        //             (fb_useable_frac * fb.height() as f32) as u32,
        //         )
        //     } else {
        //         fb.height()
        //     },
        // };
        // let btm_fb_x = if photo_is_portrait {
        //     0
        // } else {
        //     btm_fb.width() - btm_fb_crop.width
        // };
        // let btm_fb_y = if photo_is_portrait {
        //     btm_fb.height() - btm_fb_crop.height
        // } else {
        //     0
        // };

        // btm_fb = imageops::crop(
        //     &mut btm_fb,
        //     btm_fb_x,
        //     btm_fb_y,
        //     btm_fb_crop.width,
        //     btm_fb_crop.height,
        // )
        // .to_image();
        // imageops::fade_out(&mut btm_fb, fade_width, 0, fade_transition_direction);
        // imageops::overlay(
        //     &mut result_image,
        //     &btm_fb,
        //     if photo_is_portrait {
        //         overlay_x
        //     } else {
        //         overlay_x + (photo.width() - btm_fb_crop.width) as i64
        //     },
        //     overlay_y + (fit_height - btm_fb_crop.height) as i64,
        // );

        // // intermediate borders
        // let inter_fb_crop = Size {
        //     width: if photo_is_portrait {
        //         fb.width()
        //     } else {
        //         min(
        //             (0.5 * photo.width() as f32) as u32,
        //             (0.5 * fb.width() as f32) as u32,
        //         )
        //     },
        //     height: if photo_is_portrait {
        //         min(
        //             (0.5 * photo.height() as f32) as u32,
        //             (0.5 * fb.height() as f32) as u32,
        //         )
        //     } else {
        //         fb.height()
        //     },
        // };

        // let (start, end, step_size) = if photo_is_portrait {
        //     (
        //         top_fb_crop.height - fade_width,
        //         fit_height - btm_fb_crop.height + fade_width,
        //         inter_fb_crop.height as usize,
        //     )
        // } else {
        //     (
        //         top_fb_crop.width - fade_width,
        //         fit_width - btm_fb_crop.width + fade_width,
        //         inter_fb_crop.width as usize,
        //     )
        // };
        // let step_size = step_size.max(1usize);

        // crate::debug!("from {} to {} with step size {}", start, end, step_size);
        // for i in (start..=end).step_by(step_size) {
        //     let mut inter_fb = fb.clone();
        //     let (inter_fb_x, inter_fb_y, inter_fb_width, inter_fb_height) = if photo_is_portrait {
        //         (
        //             0,
        //             (0.25 * fb.height() as f32) as u32 - fade_width,
        //             inter_fb_crop.width,
        //             min(inter_fb_crop.height, end - i) + 2 * fade_width,
        //         )
        //     } else {
        //         (
        //             (0.25 * fb.width() as f32) as u32 - fade_width,
        //             0,
        //             min(inter_fb_crop.width, end - i) + 2 * fade_width,
        //             inter_fb_crop.height,
        //         )
        //     };
        //     inter_fb = imageops::crop(
        //         &mut inter_fb,
        //         inter_fb_x,
        //         inter_fb_y,
        //         inter_fb_width,
        //         inter_fb_height,
        //     )
        //     .to_image();
        //     imageops::fade_out(&mut inter_fb, fade_width, 0, fade_transition_direction);
        //     let fade_dim = if photo_is_portrait {
        //         inter_fb_height
        //     } else {
        //         inter_fb_width
        //     };
        //     imageops::fade_out(
        //         &mut inter_fb,
        //         fade_dim - fade_width,
        //         fade_dim - 1,
        //         fade_transition_direction,
        //     );
        //     imageops::overlay(
        //         &mut result_image,
        //         &inter_fb,
        //         if photo_is_portrait {
        //             overlay_x
        //         } else {
        //             overlay_x - (fade_width + i) as i64
        //         },
        //         if photo_is_portrait {
        //             overlay_y - (fade_width + i) as i64
        //         } else {
        //             overlay_y
        //         },
        //     );
        // }

        // // show the center of the final image
        // if options.preview {
        //     let highlight_color = Rgba([255, 0, 0, 50]);
        //     let mut ctr_tl = Point {
        //         x: 0,
        //         y: (size.height - size.width) / 2,
        //     };
        //     let mut ctr_br = Point {
        //         x: size.width,
        //         y: ((size.height - size.width) / 2) + size.width,
        //     };
        //     if !result_is_portrait {
        //         ctr_tl = Point {
        //             x: (size.width - size.height) / 2,
        //             y: 0,
        //         };
        //         ctr_br = Point {
        //             x: ((size.width - size.height) / 2) + size.height,
        //             y: size.height,
        //         };
        //     }
        //     imageops::fill_rect(&mut result_image, &highlight_color, ctr_tl, ctr_br);
        // };

        Ok(result_image)
        // todo!()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "borders")]
    use super::borders::BuiltinBorder;
    #[cfg(feature = "borders")]
    use super::ImageFormat;
    use super::{types, Border, BorderSource, ImageBorders, Options};
    use anyhow::Result;
    #[cfg(feature = "borders")]
    use std::io::Cursor;
    use std::path::PathBuf;
    // use tempdir::TempDir;

    // fn custom_border() -> Options {
    //     Options {
    //         output_size: Some(OutputSize {
    //             width: Some(1000),
    //             height: Some(1000),
    //         }),
    //         ..Default::default()
    //     }
    // }

    lazy_static::lazy_static! {
        pub static ref OPTIONS: Options = Options {
            output_size: types::OutputSize {
                width: Some(750),
                height: Some(750),
            },
            mode: types::Mode::FitImage,
            crop: Some(types::SidesPercent::uniform(0.05)),
            scale_factor: 0.95,
            frame_width: types::SidesPercent::uniform(0.02),
            rotate_angle: Some(types::Rotation::Rotate90),
            ..Default::default()
        };
    }

    macro_rules! format_tests {
        ($($name:ident: $values:expr,)*) => {
            $(
                #[cfg(feature = "borders")]
                #[test]
                fn $name() -> Result<()> {
                    let (infile, outfile, options) = $values;
                    let repo: PathBuf = env!("CARGO_MANIFEST_DIR").into();
                    let input = repo.join(&infile);
                    let output = repo.join(&outfile);
                    assert!(input.is_file());
                    let mut borders = ImageBorders::open(&input)?;
                    let border = BorderSource::Builtin(BuiltinBorder::Border120_1);
                    let result = borders.add_border(border, options)?;
                    result.save(Some(&output), None)?;
                    assert!(output.is_file());
                    Ok(())
                }
            )*
        }
    }

    format_tests! {
        test_open_and_save_jpg_to_jpg: (
           "samples/lowres.jpg", "testing/lowres_jpg.jpg", &OPTIONS),
        test_open_and_save_jpg_to_png: (
           "samples/lowres.jpg", "testing/lowres_jpg.png", &OPTIONS),
        test_open_and_save_jpg_to_tiff: (
           "samples/lowres.jpg", "testing/lowres_jpg.tiff", &OPTIONS),

        test_open_and_save_png_to_jpg: (
           "samples/lowres.png", "testing/lowres_png.jpg", &OPTIONS),
        test_open_and_save_png_to_png: (
           "samples/lowres.png", "testing/lowres_png.png", &OPTIONS),
        test_open_and_save_png_to_tiff: (
           "samples/lowres.png", "testing/lowres_png.tiff", &OPTIONS),

        test_open_and_save_tiff_to_jpg: (
           "samples/lowres.tiff", "testing/lowres_png.jpg", &OPTIONS),
        test_open_and_save_tiff_to_png: (
           "samples/lowres.tiff", "testing/lowres_png.png", &OPTIONS),
        test_open_and_save_tiff_to_tiff: (
           "samples/lowres.tiff", "testing/lowres_png.tiff", &OPTIONS),

        test_default_options: (
           "samples/lowres.jpg", "testing/lowres_default.jpg", &Options::default()),
    }

    #[cfg(feature = "borders")]
    #[test]
    fn test_read_write_in_memory() -> Result<()> {
        let bytes = include_bytes!("../samples/lowres.jpg");
        let input = Cursor::new(&bytes);
        let mut borders = ImageBorders::from_reader(input)?;
        let border = BorderSource::Builtin(BuiltinBorder::Border120_1);
        let result = borders.add_border(border, &OPTIONS)?;
        let mut output = Cursor::new(Vec::new());
        result.encode_to(&mut output, ImageFormat::Png, None)?;
        assert!(output.position() > 100);
        Ok(())
    }

    #[test]
    fn test_custom_border() -> Result<()> {
        let repo: PathBuf = env!("CARGO_MANIFEST_DIR").into();
        let input = repo.join("samples/lowres.jpg");
        let border_file = repo.join("samples/borders/border1.png");
        let output = repo.join("testing/lowres_custom_border.jpg");
        assert!(input.is_file());
        assert!(border_file.is_file());
        let border = BorderSource::Custom(Border::open(&border_file, None)?);
        let mut borders = ImageBorders::open(&input)?;
        let result = borders.add_border(border, &OPTIONS)?;
        result.save(Some(&output), None)?;
        assert!(output.is_file());
        Ok(())
    }

    // assert!(false);
    // let tmp_dir = TempDir::new("sample").unwrap();
    // assert!(false);
    // let tmp_dir = TempDir::new("sample").unwrap();
    // let output = tmp_dir.path().join("test_output.png");

    // let total_bytes = include_bytes!("../../experimental/audio-samples/muse_uprising.mp3");
    //     let total = Cursor::new(total_bytes.as_ref());
    // assert_eq!(
    //     CompressContentType::exclude(vec![]).should_compress(Some(content_type("image/png"))),
    //     true
    // );
}
