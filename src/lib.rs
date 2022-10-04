#![allow(warnings)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unsafe_derive_deserialize)]
// #![allow(clippy::module_name_repetitions)]

pub mod arithmetic;
pub mod border;
#[cfg(feature = "builtin")]
pub mod builtin;
pub mod debug;
pub mod defaults;
pub mod error;
pub mod imageops;
pub mod img;
pub mod options;
#[cfg(test)]
mod test;
pub mod types;
#[cfg(feature = "wasm")]
pub mod wasm;

pub use border::Border;
pub use error::Error;
pub use image::ImageFormat;
pub use imageops::FillMode;
pub use img::Image;
pub use options::*;
pub use sides::{abs::Sides, percent::Sides as SidesPercent};
pub use types::*;

use arithmetic::{
    ops::{CheckedAdd, CheckedMul, CheckedSub},
    Cast, Round,
};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ResultSize {
    output_size: Size,
    content_size: Size,
    margin: Sides,
    frame_width: Sides,
}

pub struct ImageBorders {
    images: Vec<img::Image>,
}

impl ImageBorders {
    #[inline]
    pub fn new(images: Vec<img::Image>) -> Result<ImageBorders, Error> {
        if images.is_empty() {
            Err(Error::MissingImage)
        } else {
            Ok(ImageBorders { images })
        }
    }

    #[inline]
    #[must_use]
    pub fn single(img: img::Image) -> ImageBorders {
        ImageBorders { images: vec![img] }
    }

    #[inline]
    pub fn from_reader<R: std::io::BufRead + std::io::Seek>(reader: R) -> Result<Self, Error> {
        let img = Image::from_reader(reader).unwrap();
        Ok(Self::single(img))
    }

    #[inline]
    /// Open image at file path
    ///
    /// # Errors
    ///
    /// If the image can not be opened, an error is returned
    ///
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let img = Image::open(path).unwrap();
        Ok(Self::single(img))
    }

    #[inline]
    pub fn compute_result_size(
        border: &mut Option<border::Border>,
        primary: &img::Image,
        options: &Options,
    ) -> Result<ResultSize, Error> {
        let original_content_size = match border {
            Some(ref mut border) => match options.mode {
                FitMode::Image => border.size_for(primary.size()),
                FitMode::Border => {
                    // create a new custom border
                    *border = Border::new(border.clone(), primary.size(), None)?;
                    border.size()
                }
            },
            None => primary.size(),
        };
        crate::debug!("image with border size: {}", &original_content_size);

        let scale_factor = options.scale_factor.clamp(0.0, 1.0);
        let margin_factor = f64::from(options.margin).max(0.0);

        let base = original_content_size.min_dim();
        let frame_width: Sides = options.frame_width.checked_mul(base).unwrap();
        let margin = (margin_factor * f64::from(base)).cast::<u32>().unwrap();
        let margin: Sides = Sides::uniform(margin);

        let content_size = original_content_size
            .checked_add(frame_width)
            .unwrap()
            .checked_add(margin)
            .unwrap();
        let default_output_size = content_size
            .scale_by::<_, Round>(1.0 / scale_factor)
            .unwrap();

        // set output size and do not keep aspect ratio
        let output_size = match options.output_size {
            BoundedSize {
                width: Some(width),
                height: Some(height),
            } => Ok(Size { width, height }),
            _ => default_output_size.scale_to_bounds(options.output_size, ResizeMode::Contain),
        }
        .unwrap();
        // bound output size but keep aspect ratio
        let output_size = output_size
            .scale_to_bounds(options.output_size_bounds, ResizeMode::Contain)
            .unwrap();

        let new_content_size = content_size
            .scale_to(
                output_size.checked_mul(scale_factor).unwrap(),
                ResizeMode::Contain,
            )
            .unwrap();
        let scale = f64::from(new_content_size.min_dim()) / f64::from(content_size.min_dim());
        let frame_width: Sides = frame_width.checked_mul(scale).unwrap();
        let margin: Sides = margin.checked_mul(scale).unwrap();
        // crate::debug!(&frame_width);
        // crate::debug!(&margin);

        Ok(ResultSize {
            content_size: new_content_size,
            margin,
            frame_width,
            output_size,
        })
    }

    #[inline]
    /// Add (optional) border to image
    ///
    /// # Errors
    ///
    /// If the border can not be added, an error is returned.
    ///
    pub fn add_border(
        &mut self,
        border: Option<border::Kind>,
        options: &Options,
    ) -> Result<img::Image, Error> {
        // prepare images
        let mut images: Vec<img::Image> = self.images.clone();
        let primary = images.get_mut(0).unwrap(); // .ok_or(Error::MissingImage)?;
        primary.rotate(&options.image_rotation);
        if let Some(crop_percent) = options.crop {
            let crop = crop_percent.checked_mul(primary.size()).unwrap();
            primary.crop_sides(crop);
        };

        // prepare the border for the primary image
        let mut border = match border {
            Some(border) => {
                let mut border = border.into_border().unwrap();
                border.rotate_to_orientation(primary.orientation()).unwrap();
                border.rotate(&options.border_rotation).unwrap();
                Some(border)
            }
            None => None,
        };

        let result_size = Self::compute_result_size(&mut border, &*primary, options)?;
        crate::debug!(&result_size);

        // create new result image
        let mut result_image = img::Image::with_size(result_size.output_size);
        result_image.path = primary.path.clone();

        let background_color = options.background_color.unwrap_or(if options.preview {
            Color::gray()
        } else {
            Color::white()
        });
        result_image.fill(background_color, FillMode::Set);

        let content_rect = result_size
            .output_size
            .center(result_size.content_size)
            .unwrap();
        crate::debug!(&content_rect);

        #[cfg(debug_assertions)]
        result_image
            .fill_rect(
                Color::rgba(0, 0, 255, 100),
                &content_rect,
                // content_rect.top_left(),
                // content_rect.size()?,
                FillMode::Blend,
            )
            .unwrap();

        let content_rect_sub_margin = content_rect.checked_sub(result_size.margin).unwrap();
        result_image
            .fill_rect(
                options.frame_color,
                &content_rect_sub_margin,
                // content_rect_sub_margin.top_left(),
                // content_rect_sub_margin.size()?,
                FillMode::Set,
            )
            .unwrap();

        let border_rect = content_rect_sub_margin
            .checked_sub(result_size.frame_width)
            .unwrap();
        crate::debug!(&border_rect);
        let border_size = border_rect.size().unwrap();

        #[cfg(debug_assertions)]
        result_image.fill_rect(
            Color::rgba(0, 255, 0, 100),
            &border_rect,
            // border_rect.top_left(),
            // border_size,
            FillMode::Blend,
        );
        let default_component = Rect::new(Point::origin(), border_size).unwrap();

        crate::debug!("overlay content");
        match options.mode {
            FitMode::Image => {
                let default_component = vec![default_component];
                let components = match border {
                    Some(ref mut border) => {
                        border.resize_and_crop(border_size, ResizeMode::Contain)?;

                        let default_image = primary.clone();
                        images.resize(border.transparent_components().len(), default_image);
                        border
                            .transparent_components()
                            .iter()
                            .zip(images.iter_mut())
                    }
                    None => default_component.iter().zip(images.iter_mut()),
                };

                for (c, image) in components {
                    crate::debug!("drawing {:?}", &c);
                    let mut image_rect = c.checked_add(border_rect.top_left()).unwrap();
                    image_rect = image_rect.padded(3).unwrap();
                    image_rect = image_rect.clamp(&border_rect);
                    let image_size = image_rect.size().unwrap();

                    let center_offset = image_rect.center_offset_to(&border_rect).unwrap();
                    image.resize_and_crop(
                        image_size,
                        ResizeMode::Cover,
                        CropMode::Custom {
                            x: center_offset.x,
                            y: center_offset.y,
                        },
                    );

                    result_image.overlay(image, image_rect.top_left());
                }

                if let Some(border) = border {
                    result_image.overlay(&*border, border_rect.top_left());
                }
            }
            FitMode::Border => {
                let c = match border {
                    Some(ref mut border) => {
                        let border_size = border_rect.size().unwrap();
                        border.resize_and_crop(border_size, ResizeMode::Contain)?;
                        border.content_rect()
                    }
                    None => &default_component,
                };

                let mut image_rect = c.checked_add(border_rect.top_left()).unwrap();
                image_rect = image_rect.padded(3).unwrap();
                image_rect = image_rect.clamp(&border_rect);
                let image_size = image_rect.size().unwrap();

                primary.resize_and_crop(image_size, ResizeMode::Cover, CropMode::Center);

                result_image.overlay(&*primary, image_rect.top_left());
                if let Some(border) = border {
                    result_image.overlay(&*border, border_rect.top_left());
                }
            }
        };

        if options.preview {
            let preview_size = Size {
                width: result_size.output_size.min_dim(),
                height: result_size.output_size.min_dim(),
            };
            let preview_rect = result_size.output_size.center(preview_size).unwrap();
            result_image.fill_rect(
                Color::rgba(255, 0, 0, 50),
                &preview_rect,
                // preview_rect.top_left(),
                // preview_rect.size()?,
                FillMode::Blend,
            );
        }

        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::border::{self, Border};
    #[cfg(feature = "builtin")]
    use super::{builtin, ImageFormat};
    use super::{types, ImageBorders, Options};
    use anyhow::Result;
    #[cfg(feature = "builtin")]
    use std::io::Cursor;
    use std::path::PathBuf;

    lazy_static::lazy_static! {
        pub static ref OPTIONS: Options = Options {
            output_size: types::BoundedSize {
                width: Some(750),
                height: Some(750),
            },
            mode: types::FitMode::Image,
            crop: Some(types::sides::percent::Sides::uniform(0.05)),
            scale_factor: 0.95,
            frame_width: types::sides::percent::Sides::uniform(0.02),
            image_rotation: types::Rotation::Rotate90,
            ..Default::default()
        };
    }

    macro_rules! format_tests {
        ($($name:ident: $values:expr,)*) => {
            $(
                #[cfg(feature = "builtin")]
                #[test]
                fn $name() -> Result<()> {
                    let (infile, outfile, options) = $values;
                    let repo: PathBuf = env!("CARGO_MANIFEST_DIR").into();
                    let input = repo.join(&infile);
                    let output = repo.join(&outfile);
                    assert!(input.is_file());
                    let mut borders = ImageBorders::open(&input)?;
                    let border = border::Kind::Builtin(builtin::Builtin::Border120_1);
                    let result = borders.add_border(Some(border), options)?;
                    result.save_with_filename(&output, None)?;
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

    #[cfg(feature = "builtin")]
    #[test]
    fn test_read_write_in_memory() -> Result<()> {
        let bytes = include_bytes!("../samples/lowres.jpg");
        let input = Cursor::new(&bytes);
        let mut borders = ImageBorders::from_reader(input)?;
        let border = border::Kind::Builtin(builtin::Builtin::Border120_1);
        let result = borders.add_border(Some(border), &OPTIONS)?;
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
        let border = border::Kind::Custom(Border::open(&border_file, None)?);
        let mut borders = ImageBorders::open(&input)?;
        let result = borders.add_border(Some(border), &OPTIONS)?;
        result.save_with_filename(&output, None)?;
        assert!(output.is_file());
        Ok(())
    }
}
