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
pub use imageops::FillMode;
pub use img::Image;
pub use options::*;
pub use types::*;

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
        primary.rotate(options.image_rotation);
        if let Some(crop_percent) = options.crop {
            let crop = crop_percent * primary.size();
            primary.crop_sides(crop);
        };

        // prepare the border for the primary image
        let mut border = match border {
            Some(border) => {
                let mut border = border.into_border()?;
                border.rotate_to_orientation(primary.orientation())?;
                border.rotate(options.border_rotation)?;
                Some(border)
            }
            None => None,
        };

        let original_content_size = match border {
            Some(ref mut border) => match options.mode {
                Mode::FitImage => border.size_for(primary.size()),
                Mode::FitBorder => {
                    // create a new custom border
                    *border = Border::new(border.clone(), primary.size(), None)?;
                    border.size()
                }
            },
            None => primary.size(),
        };
        crate::debug!("image with border size: {}", &original_content_size);

        let scale_factor = utils::clamp(options.scale_factor, 0.0, 1.0);
        let margin_factor = options.margin.max(0.0);

        let base = original_content_size.min_dim();
        let frame_width: Sides = options.frame_width * base;
        let margin: Sides = Sides::uniform((margin_factor * base as f32) as u32);
        let content_size = original_content_size + frame_width + margin;
        let default_output_size = content_size * (1.0 / scale_factor);

        // set output size and do not keep aspect ratio
        let output_size = match options.output_size {
            OutputSize {
                width: Some(width),
                height: Some(height),
            } => Size { width, height },
            _ => default_output_size.scale_to_bounds(options.output_size, ResizeMode::Contain),
        };
        // bound output size but keep aspect ratio
        let output_size =
            output_size.scale_to_bounds(options.output_size_bounds, ResizeMode::Contain);

        // create new result image
        let mut result_image = img::Image::with_size(output_size);
        result_image.path = primary.path.clone();

        let background_color = options.background_color.unwrap_or(if options.preview {
            Color::gray()
        } else {
            Color::white()
        });
        result_image.fill(background_color, FillMode::Set);

        let new_content_size =
            content_size.scale_to(output_size * scale_factor, ResizeMode::Contain);
        let scale = new_content_size.min_dim() as f64 / content_size.min_dim() as f64;
        let frame_width = frame_width * scale;
        let margin = margin * scale;
        crate::debug!(&frame_width);
        crate::debug!(&margin);

        let content_rect = output_size.center(new_content_size);
        crate::debug!(&content_rect);

        #[cfg(debug_assertions)]
        result_image.fill_rect(
            Color::rgba(0, 0, 255, 100),
            content_rect.top_left(),
            content_rect.size(),
            FillMode::Blend,
        );

        result_image.fill_rect(
            options.frame_color,
            (content_rect - margin).top_left(),
            (content_rect - margin).size(),
            FillMode::Set,
        );

        let border_rect = content_rect - margin - frame_width;
        crate::debug!(&border_rect);

        #[cfg(debug_assertions)]
        result_image.fill_rect(
            Color::rgba(0, 255, 0, 100),
            border_rect.top_left(),
            border_rect.size(),
            FillMode::Blend,
        );
        let default_component = Rect::new(Point::origin(), border_rect.size());

        crate::debug!("overlay content");
        match options.mode {
            Mode::FitImage => {
                let default_component = vec![default_component];
                let components = match border {
                    Some(ref mut border) => {
                        border.resize_to_fit(border_rect.size(), ResizeMode::Contain)?;

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
                    let mut image_rect = *c + border_rect.top_left();
                    image_rect = image_rect.extend(3);
                    image_rect = image_rect.clip_to(&border_rect);

                    let crop_mode = image_rect.crop_mode(&border_rect);
                    image.resize_to_fit(image_rect.size(), ResizeMode::Cover, crop_mode);

                    result_image.overlay(image.as_ref(), image_rect.top_left());
                }

                if let Some(border) = border {
                    result_image.overlay(border.as_ref(), border_rect.top_left());
                }
            }
            Mode::FitBorder => {
                let c = match border {
                    Some(ref mut border) => {
                        border.resize_to_fit(border_rect.size(), ResizeMode::Contain)?;
                        border.content_rect()
                    }
                    None => &default_component,
                };

                let mut image_rect = *c + border_rect.top_left();
                image_rect = image_rect.extend(3);
                image_rect = image_rect.clip_to(&border_rect);

                primary.resize_to_fit(image_rect.size(), ResizeMode::Cover, CropMode::Center);

                result_image.overlay(primary.as_ref(), image_rect.top_left());
                if let Some(border) = border {
                    result_image.overlay(border.as_ref(), border_rect.top_left());
                }
            }
        };

        if options.preview {
            let preview_size = Size {
                width: output_size.min(),
                height: output_size.min(),
            };
            let preview_rect = output_size.center(preview_size);
            result_image.fill_rect(
                Color::rgba(255, 0, 0, 50),
                preview_rect.top_left(),
                preview_rect.size(),
                FillMode::Blend,
            );
        }

        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "borders")]
    use super::borders::BuiltinBorder;
    use super::types::*;
    #[cfg(feature = "borders")]
    use super::ImageFormat;
    use super::{Border, BorderSource, ImageBorders, Options};
    use anyhow::Result;
    #[cfg(feature = "borders")]
    use std::io::Cursor;
    use std::path::PathBuf;

    lazy_static::lazy_static! {
        pub static ref OPTIONS: Options = Options {
            output_size: OutputSize {
                width: Some(750),
                height: Some(750),
            },
            mode: Mode::FitImage,
            crop: Some(SidesPercent::uniform(0.05)),
            scale_factor: 0.95,
            frame_width: SidesPercent::uniform(0.02),
            image_rotation: Some(Rotation::Rotate90),
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
                    let result = borders.add_border(Some(border), options)?;
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
        let border = BorderSource::Custom(Border::open(&border_file, None)?);
        let mut borders = ImageBorders::open(&input)?;
        let result = borders.add_border(Some(border), &OPTIONS)?;
        result.save(Some(&output), None)?;
        assert!(output.is_file());
        Ok(())
    }
}
