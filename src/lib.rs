// #![allow(warnings)]
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
pub use image::ImageFormat;
pub use imageops::FillMode;
pub use img::Image;
pub use options::*;
pub use sides::{abs::Sides, percent::Sides as SidesPercent};
pub use types::*;

use arithmetic::{
    ops::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
    Cast, Round,
};
use serde::Serialize;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, PartialEq, Clone, Debug)]
pub struct ResultSize {
    output_size: Size,
    content_size: Size,
    margins: Sides,
    frame_width: Sides,
    scale_factor: f32,
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
    pub fn from_reader(reader: impl std::io::BufRead + std::io::Seek) -> Result<Self, Error> {
        let img = Image::from_reader(reader)?;
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
        let img = Image::open(path)?;
        Ok(Self::single(img))
    }

    #[inline]
    /// Add (optional) border to image
    ///
    /// # Errors
    ///
    /// If the border can not be added, an error is returned.
    ///
    pub fn render(
        &mut self,
        border_kind: Option<border::Kind>,
        options: &Options,
    ) -> Result<img::Image, RenderError> {
        let mut images: Vec<img::Image> = self.images.clone();
        let primary = images.get_mut(0).ok_or(RenderError::MissingImage)?;

        prepare_primary(primary, options)?;
        let mut border = border_for_primary(border_kind, primary, options)?;

        let result_size = compute_result_size(&border, &*primary, options)?;
        debug!(&result_size);

        // create new result image
        let mut result_image = img::Image {
            path: primary.path.clone(),
            ..img::Image::with_size(result_size.output_size)
        };

        result_image
            .fill(options.background_color(), FillMode::Set)
            .map_err(img::Error::from)?;

        let content_rect = result_size
            .output_size
            .center(result_size.content_size)
            .map_err(|err| error::Arithmetic {
                msg: "failed to center content size".to_string(),
                source: err.into(),
            })?;
        debug!(&content_rect);

        #[cfg(feature = "debug")]
        {
            let blue = Color::rgba(0, 0, 255, 100);
            result_image
                .fill_rect(blue, &content_rect, FillMode::Blend)
                .map_err(img::Error::from)?;

            let black = Color::black();
            draw_text_mut(
                &mut result_image,
                "content size",
                black,
                content_rect.top_left(),
            )?;
        }

        let content_rect_sub_margins =
            content_rect
                .checked_sub(result_size.margins)
                .map_err(|err| error::Arithmetic {
                    msg: "failed to compute content rect without margins".into(),
                    source: err.into(),
                })?;

        result_image
            .fill_rect(
                options.frame_color,
                &content_rect_sub_margins,
                FillMode::Set,
            )
            .map_err(img::Error::from)?;

        let border_rect = content_rect_sub_margins
            .checked_sub(result_size.frame_width)
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute border rect".into(),
                source: err.into(),
            })?;
        debug!(&border_rect);
        let border_size = border_rect.size().map_err(|err| error::Arithmetic {
            msg: "failed to compute border size".into(),
            source: err.into(),
        })?;

        #[cfg(feature = "debug")]
        {
            let green = Color::rgba(0, 255, 0, 100);
            result_image
                .fill_rect(green, &border_rect, FillMode::Blend)
                .map_err(img::Error::from)?;
        }
        let primary_component = Rect::from(border_size);

        debug!("overlay content");
        match options.mode {
            FitMode::Image => {
                let primary_component = vec![primary_component];
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
                    None => primary_component.iter().zip(images.iter_mut()),
                };

                for (idx, (component_rect, component)) in components.enumerate() {
                    draw_component(&mut result_image, component, component_rect, &border_rect)
                        .map_err(|err| RenderComponentError {
                            idx,
                            rect: *component_rect,
                            size: component.size(),
                            source: err.into(),
                        })?;
                }

                if let Some(border) = border {
                    result_image.overlay(&*border, border_rect.top_left());
                }
            }
            FitMode::Border => {
                let primary_component_rect = match border {
                    Some(ref mut border) => {
                        border.resize_and_crop(border_size, ResizeMode::Contain)?;
                        border.content_rect().map_err(border::Error::from)?
                    }
                    None => &primary_component,
                };

                draw_component(
                    &mut result_image,
                    primary,
                    primary_component_rect,
                    &border_rect,
                )
                .map_err(|err| RenderComponentError {
                    idx: 0,
                    rect: *primary_component_rect,
                    size: primary.size(),
                    source: err.into(),
                })?;
                if let Some(border) = border {
                    result_image.overlay(&*border, border_rect.top_left());
                }
            }
        };

        if options.preview {
            overlay_visible_area(&mut result_image)?;
        }

        Ok(result_image)
    }
}

#[inline]
fn compute_pre_result_size(
    border: &Option<border::Border>,
    primary: &img::Image,
    options: &Options,
) -> Result<ResultSize, ResultSizeError> {
    let scale_factor = options.scale_factor.clamp(0.0, 1.0);
    let margin_factor = f64::from(options.margin).max(0.0);

    let original_content_size = match border {
        Some(border) => match options.mode {
            FitMode::Image => border.size_for(primary.size())?,
            FitMode::Border => border.size(),
        },
        None => primary.size(),
    };
    debug!(&primary.size());
    debug!(&original_content_size);

    let base = original_content_size.min_dim();

    let frame_width: Sides =
        options
            .frame_width
            .checked_mul(base)
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute original frame width".to_string(),
                source: err.into(),
            })?;
    debug!(&frame_width);

    let margin = (|| {
        let margin = CheckedMul::checked_mul(margin_factor, f64::from(base))?;
        let margin = margin.cast::<u32>()?;
        Ok::<_, arithmetic::Error>(margin)
    })();
    let margin = margin.map_err(|err| error::Arithmetic {
        msg: "failed to compute original margin width".to_string(),
        source: err,
    })?;
    let margins = Sides::uniform(margin);
    debug!(&margins);

    let content_size = original_content_size
        .checked_add(frame_width)
        .and_then(|size| size.checked_add(margins))
        .map_err(|err| error::Arithmetic {
            msg: "failed to compute content size".to_string(),
            source: err.into(),
        })?;
    debug!(&content_size);

    let default_output_size = content_size
        .scale_by::<_, Round>(1.0 / scale_factor)
        .map_err(|err| error::Arithmetic {
            msg: "failed to compute default output size".to_string(),
            source: err.into(),
        })?;
    debug!(&default_output_size);

    // set output size and do not keep aspect ratio
    let output_size = match options.output_size {
        BoundedSize {
            width: Some(width),
            height: Some(height),
        } => Size { width, height },
        _ => default_output_size
            .scale_to_bounds(options.output_size, ResizeMode::Contain)
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute output size".to_string(),
                source: err.into(),
            })?,
    };
    // bound output size but keep aspect ratio
    let output_size = output_size
        .scale_to_bounds(options.output_size_bounds, ResizeMode::Contain)
        .map_err(|err| error::Arithmetic {
            msg: "failed to bound output size".to_string(),
            source: err.into(),
        })?;

    debug!(&output_size);
    Ok(ResultSize {
        output_size,
        content_size,
        margins,
        frame_width,
        scale_factor,
    })
}

#[inline]
fn compute_result_size(
    border: &Option<border::Border>,
    primary: &img::Image,
    options: &Options,
) -> Result<ResultSize, ResultSizeError> {
    let pre_result_size = compute_pre_result_size(border, primary, options)?;

    let post_content_size_scale = pre_result_size
        .output_size
        .checked_mul(pre_result_size.scale_factor)
        .map_err(|err| error::Arithmetic {
            msg: "failed to compute scaled content size".into(),
            source: err.into(),
        })?;

    let pre_content_size = pre_result_size.content_size;
    let post_content_size = pre_content_size
        .scale_to(post_content_size_scale, ResizeMode::Contain)
        .map_err(|err| error::Arithmetic {
            msg: "failed to compute scaled content size".into(),
            source: err.into(),
        })?;
    debug!(&post_content_size);

    let pre_base = f64::from(pre_content_size.min_dim());
    let post_base = f64::from(post_content_size.min_dim());
    let scale = CheckedDiv::checked_div(post_base, pre_base).map_err(|err| error::Arithmetic {
        msg: "failed to compute post base scale".into(),
        source: err.into(),
    })?;

    debug!(&scale);

    let frame_width = pre_result_size
        .frame_width
        .checked_mul(scale)
        .map_err(|err| error::Arithmetic {
            msg: "failed to compute scaled frame width".into(),
            source: err.into(),
        })?;
    debug!(&frame_width);

    let margins = pre_result_size
        .margins
        .checked_mul(scale)
        .map_err(|err| error::Arithmetic {
            msg: "failed to compute scaled margins".into(),
            source: err.into(),
        })?;
    debug!(&margins);

    Ok(ResultSize {
        content_size: post_content_size,
        margins,
        frame_width,
        ..pre_result_size
    })
}

#[inline]
fn border_for_primary(
    border_kind: Option<border::Kind>,
    primary: &img::Image,
    options: &Options,
) -> Result<Option<Border>, RenderError> {
    let mut border = match border_kind {
        Some(border_kind) => {
            // prepare the border for the primary image
            let mut border = border_kind.into_border()?;
            border.rotate_to_orientation(primary.orientation())?;
            border.rotate(&options.border_rotation)?;
            Some(border)
        }
        None => None,
    };

    if let Some(ref mut border) = border {
        if let FitMode::Border = options.mode {
            *border = Border::custom(border.clone(), primary.size(), None)
                .map_err(border::Error::from)?;
        }
    }
    Ok(border)
}

#[inline]
fn prepare_primary(primary: &mut img::Image, options: &Options) -> Result<(), PreparePrimaryError> {
    primary.rotate(&options.image_rotation);
    if let Some(crop_percent) = options.crop {
        let crop = crop_percent
            .checked_mul(primary.size())
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute crop from relative crop".to_string(),
                source: err.into(),
            })?;
        primary
            .crop_sides(crop)
            .map_err(img::CropError::from)
            .map_err(img::Error::from)?;
    };
    Ok(())
}

#[cfg(feature = "debug")]
fn draw_text_mut(
    image: &mut img::Image,
    text: &str,
    color: impl Into<image::Rgba<u8>>,
    top_left: Point,
) -> Result<(), RenderError> {
    use rusttype::{Font, Scale};

    lazy_static::lazy_static! {
        pub static ref INTER: Font<'static> = {
            let font_data = include_bytes!("../fonts/Inter-Regular.ttf");
            Font::try_from_bytes(font_data).expect("read font bytes")
        };
    };

    let top_left = (|| {
        let top_left = top_left.checked_add(Point { x: 3, y: 3 })?;
        let x = top_left.x.cast::<i32>()?;
        let y = top_left.y.cast::<i32>()?;
        Ok::<_, arithmetic::Error>((x, y))
    })();
    let (x, y) = top_left.map_err(|err| error::Arithmetic {
        msg: format!("failed to compute top left point for text `{}`", text),
        source: err,
    })?;

    let scale = image
        .size()
        .max_dim()
        .cast::<f32>()
        .map_err(arithmetic::Error::from)
        .and_then(|max_dim| CheckedMul::checked_mul(max_dim, 0.03).map_err(arithmetic::Error::from))
        .map_err(|err| error::Arithmetic {
            msg: "failed to compute text scale".into(),
            source: err,
        })?;
    let scale = Scale::uniform(scale);
    imageproc::drawing::draw_text_mut(&mut **image, color.into(), x, y, scale, &INTER, text);
    Ok(())
}

#[inline]
fn overlay_visible_area(image: &mut img::Image) -> Result<(), RenderError> {
    let size = image.size();
    let preview_size = Size {
        width: size.min_dim(),
        height: size.min_dim(),
    };
    let preview_rect = size.center(preview_size).map_err(|err| error::Arithmetic {
        msg: "failed to compute centered preview rect".into(),
        source: err.into(),
    })?;

    let transparent_red = Color::rgba(255, 0, 0, 50);
    image
        .fill_rect(transparent_red, &preview_rect, FillMode::Blend)
        .map_err(img::Error::from)?;
    Ok(())
}

#[inline]
fn draw_component(
    image: &mut img::Image,
    component: &mut img::Image,
    component_rect: &Rect,
    border_rect: &Rect,
) -> Result<(), RenderError> {
    debug!("drawing", &component_rect);

    let component_rect = (|| {
        let mut component_rect = component_rect.checked_add(border_rect.top_left())?;
        component_rect = component_rect.padded(3)?;
        component_rect = component_rect.clamp(border_rect);
        Ok::<_, arithmetic::Error>(component_rect)
    })();
    let component_rect = component_rect.map_err(|err| error::Arithmetic {
        msg: "failed to compute component rect".into(),
        source: err,
    })?;
    let component_size = component_rect.size().map_err(|err| error::Arithmetic {
        msg: "failed to compute component rect size".into(),
        source: err.into(),
    })?;

    let center_offset = component_rect
        .center_offset_to(border_rect)
        .map_err(|err| error::Arithmetic {
            msg: "failed to compute center offset of component".into(),
            source: err.into(),
        })?;

    #[cfg(feature = "debug")]
    {
        let red = Color::rgba(255, 255, 0, 100);
        image
            .fill_rect(red, &component_rect, FillMode::Blend)
            .map_err(img::Error::from)?;

        let mut component = component.clone();
        component
            .clip_alpha(&Rect::from(component.size()), 0, 60)
            .map_err(img::Error::from)?;
        component
            .resize(component_size, ResizeMode::Cover)
            .map_err(img::Error::from)?;
        let offset = component_size
            .center(component.size())
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute center offset of uncropped component".into(),
                source: err.into(),
            })?;

        debug!(&offset);

        let uncropped_component_top_left = component_rect
            .top_left()
            .checked_add(offset.top_left())
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute top left of uncropped component".into(),
                source: err.into(),
            })?;

        image.overlay(&component, uncropped_component_top_left);
    }

    component
        .resize_and_crop(
            component_size,
            ResizeMode::Cover,
            CropMode::Custom {
                x: center_offset.x,
                y: center_offset.y,
            },
        )
        .map_err(img::Error::from)?;
    assert_eq!(component_size, component.size());

    image.overlay(component, component_rect.top_left());
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum PreparePrimaryError {
    #[error(transparent)]
    Arithmetic(#[from] error::Arithmetic),

    #[error(transparent)]
    Image(#[from] img::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ResultSizeError {
    #[error(transparent)]
    Arithmetic(#[from] error::Arithmetic),

    #[error(transparent)]
    Border(#[from] border::Error),
}

#[derive(thiserror::Error, Debug)]
#[error("failed to render component {idx} with size {size:#?} in {rect:#?}")]
pub struct RenderComponentError {
    idx: usize,
    rect: Rect,
    size: Size,
    source: Box<RenderError>,
}

#[derive(thiserror::Error, Debug)]
pub enum RenderError {
    #[error("missing input image")]
    MissingImage,

    #[error(transparent)]
    Image(#[from] img::Error),

    #[error(transparent)]
    RenderComponent(#[from] RenderComponentError),

    #[error(transparent)]
    Arithmetic(#[from] error::Arithmetic),

    #[error("failed to compute result size")]
    ResultSize(
        #[from]
        #[source]
        ResultSizeError,
    ),

    #[error("failed to prepare primary image")]
    PreparePrimary(
        #[from]
        #[source]
        PreparePrimaryError,
    ),

    #[error(transparent)]
    Border(#[from] border::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("missing input image")]
    MissingImage,

    #[error("failed to read image")]
    Read(
        #[from]
        #[source]
        img::ReadError,
    ),

    #[error(transparent)]
    Image(#[from] img::Error),

    #[error("render error")]
    Render(
        #[from]
        #[source]
        RenderError,
    ),
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
                width: Some(2000),
                height: Some(2000),
            },
            // mode: types::FitMode::Image,
            mode: types::FitMode::Border,
            crop: Some(types::sides::percent::Sides::uniform(0.05)),
            scale_factor: 0.90,
            // frame_width: types::sides::percent::Sides::uniform(0.02),
            frame_width: types::sides::percent::Sides::uniform(0.1),
            margin: 0.1,
            // image_rotation: types::Rotation::Rotate90,
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
                    let result = borders.render(Some(border), options)?;
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
        let result = borders.render(Some(border), &OPTIONS)?;
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
        let result = borders.render(Some(border), &OPTIONS)?;
        result.save_with_filename(&output, None)?;
        assert!(output.is_file());
        Ok(())
    }
}
