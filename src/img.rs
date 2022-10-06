use super::arithmetic::{self, ops::CheckedSub, Cast, CastError};
use super::types::{self, sides::abs::Sides, Point, Rect, Size};
use super::{defaults, error, imageops};
use crate::{debug, debug::Instant};
pub use image::ImageFormat;
use std::borrow::Borrow;
use std::fs;
use std::io::{BufReader, Seek};
use std::path::{Path, PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum ReadErrorSource {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::error::ImageError),
}

#[derive(thiserror::Error, Debug)]
#[error("failed to read image from path {path:?}")]
pub struct ReadError {
    path: Option<PathBuf>,
    source: ReadErrorSource,
}

#[derive(thiserror::Error, Debug)]
pub enum SaveErrorSource {
    #[error("missing output file path")]
    MissingOutputPath,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::error::ImageError),
}

#[derive(thiserror::Error, Debug)]
#[error("failed to save image to {path:?} with format {format:?} and quality {quality:?}")]
pub struct SaveError {
    path: Option<PathBuf>,
    format: Option<ImageFormat>,
    quality: Option<u8>,
    source: SaveErrorSource,
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("failed to fill {rect:#?} of image with size {size:#?}")]
pub struct FillError {
    rect: Rect,
    size: Size,
    source: SubImageError,
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum FadeErrorSource {
    #[error(transparent)]
    SubImage(#[from] SubImageError),

    #[error(transparent)]
    Fade(#[from] imageops::FadeError),
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("failed to fade out image with size {:#?} from {start:#?} to {end:#?} along {axis}")]
pub struct FadeError {
    size: Size,
    start: Point,
    end: Point,
    axis: types::Axis,
    source: FadeErrorSource,
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("failed to resize image with size {size:#?} to {target:#?} with mode {mode:?}")]
pub struct ResizeError {
    size: Size,
    target: Size,
    mode: super::ResizeMode,
    source: types::size::ScaleToError,
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum CropErrorSource {
    #[error(transparent)]
    CropToFit(#[from] types::size::CropToFitError),

    #[error(transparent)]
    SubImage(#[from] SubImageError),

    #[error(transparent)]
    Sides(#[from] types::rect::SubSidesError),

    #[error(transparent)]
    CropRect(#[from] Box<CropRectError>),
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("failed to crop image with size {size} to fit {target}")]
pub struct CropToFitError {
    size: Size,
    target: Size,
    source: CropErrorSource,
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("failed to crop image with size {size} to {rect}")]
pub struct CropRectError {
    size: Size,
    rect: Rect,
    source: CropErrorSource,
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("failed to crop image with size {size} by {sides}")]
pub struct CropSidesError {
    size: Size,
    sides: Sides,
    source: CropErrorSource,
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum CropError {
    #[error(transparent)]
    CropSides(#[from] CropSidesError),

    #[error(transparent)]
    CropRect(#[from] CropRectError),

    #[error(transparent)]
    CropToFit(#[from] CropToFitError),
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum ResizeAndCropError {
    #[error(transparent)]
    Resize(#[from] ResizeError),
    #[error(transparent)]
    Crop(#[from] CropToFitError),
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum SubImageErrorSource {
    #[error(transparent)]
    OutOfBounds(#[from] OutOfBoundsError),

    #[error(transparent)]
    Cast(#[from] CastError<i64, u32>),
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("failed to get sub image {rect} for image with size {size}")]
pub struct SubImageError {
    size: Size,
    rect: Rect,
    source: SubImageErrorSource,
}

#[derive(thiserror::Error, PartialEq, Clone, Debug)]
#[error("point {point} exceeds image bounds {bounds}")]
pub struct OutOfBoundsError {
    point: Point,
    bounds: Rect,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // #[error("failed to resize image")]
    #[error(transparent)]
    Resize(#[from] ResizeError),

    //     #[error("failed to crop image")]
    #[error(transparent)]
    Crop(#[from] CropError),

    #[error(transparent)]
    ResizeAndCrop(#[from] ResizeAndCropError),

    // #[error("failed to get subview of image")]
    #[error(transparent)]
    SubImage(
        #[from]
        // #[source]
        SubImageError,
    ),

    // #[error("failed to fill image")]
    #[error(transparent)]
    Fill(
        #[from]
        // #[source]
        FillError,
    ),

    // #[error("failed to fade out image")]
    #[error(transparent)]
    FadeOut(
        #[from]
        // #[source]
        imageops::FadeError,
    ),

    // #[error("failed to read image")]
    #[error(transparent)]
    Read(
        #[from]
        // #[source]
        ReadError,
    ),

    // #[error("failed to save image")]
    #[error(transparent)]
    Save(
        #[from]
        // #[source]
        SaveError,
    ),
}

#[derive(Clone)]
pub struct Image {
    pub(crate) inner: image::RgbaImage,
    pub(crate) path: Option<PathBuf>,
}

impl std::ops::Deref for Image {
    type Target = image::RgbaImage;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Image {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Image {
    #[inline]
    #[must_use]
    pub fn size(&self) -> Size {
        Size::from(&self.inner)
    }

    #[inline]
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let inner = image::RgbaImage::new(width, height);
        Self { inner, path: None }
    }

    #[inline]
    #[must_use]
    pub fn with_size(size: impl Into<Size>) -> Self {
        let size = size.into();
        Self::new(size.width, size.height)
    }

    #[inline]
    #[must_use]
    pub fn from_image(image: &image::DynamicImage) -> Self {
        let inner = image.to_rgba8();
        Self { inner, path: None }
    }

    #[inline]
    pub fn from_reader(reader: impl std::io::BufRead + std::io::Seek) -> Result<Self, ReadError> {
        match (|| {
            let reader = image::io::Reader::new(reader).with_guessed_format()?;
            let inner = reader.decode()?.to_rgba8();
            let image = Self { inner, path: None };
            Ok::<Self, ReadErrorSource>(image)
        })() {
            Ok(image) => Ok(image),
            Err(err) => Err(ReadError {
                path: None,
                source: err.into(),
            }),
        }
    }

    #[inline]
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, ReadError> {
        let path = path.into();
        let file = fs::OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|err| ReadError {
                path: Some(path.clone()),
                source: err.into(),
            })?;
        let reader = BufReader::new(&file);
        let image = Self::from_reader(reader).map_err(|err| ReadError {
            path: Some(path.clone()),
            source: err.source,
        })?;
        Ok(Self {
            path: Some(path.clone()),
            ..image
        })
    }

    #[inline]
    #[must_use]
    pub fn is_portrait(&self) -> bool {
        self.size().is_portrait()
    }

    #[inline]
    #[must_use]
    pub fn orientation(&self) -> super::Orientation {
        self.size().orientation()
    }

    #[inline]
    pub fn sub_image(
        &mut self,
        rect: &Rect,
    ) -> Result<image::SubImage<&mut image::RgbaImage>, SubImageError> {
        pub struct SubImageRect {
            top: u32,
            left: u32,
            bottom: u32,
            right: u32,
        }

        let image_rect: Rect = self.size().into();
        match (|| {
            if !image_rect.contains(&rect.top_left()) {
                return Err(SubImageErrorSource::OutOfBounds(OutOfBoundsError {
                    bounds: image_rect,
                    point: rect.top_left(),
                }));
            }

            if !image_rect.contains(&rect.bottom_right()) {
                return Err(SubImageErrorSource::OutOfBounds(OutOfBoundsError {
                    bounds: image_rect,
                    point: rect.bottom_right(),
                }));
            }

            let sub_image_rect = SubImageRect {
                top: rect.top.cast::<u32>()?,
                left: rect.left.cast::<u32>()?,
                bottom: rect.bottom.cast::<u32>()?,
                right: rect.right.cast::<u32>()?,
            };
            Ok::<_, SubImageErrorSource>(sub_image_rect)
        })() {
            Ok(sub_image_rect) => {
                use image::GenericImage;
                let x = sub_image_rect.left;
                let y = sub_image_rect.top;
                let width = sub_image_rect.right - sub_image_rect.left;
                let height = sub_image_rect.bottom - sub_image_rect.top;
                Ok(self.inner.sub_image(x, y, width, height))
            }
            Err(err) => Err(SubImageError {
                rect: *rect,
                size: self.size(),
                source: err.into(),
            }),
        }
    }

    #[inline]
    pub fn fill(
        &mut self,
        color: impl Into<image::Rgba<u8>>,
        mode: imageops::FillMode,
    ) -> Result<(), FillError> {
        let rect: Rect = self.size().into();
        self.fill_rect(color.into(), &rect, mode)
    }

    #[inline]
    pub fn clip_alpha(&mut self, rect: &Rect, min: u8, max: u8) -> Result<(), SubImageError> {
        let sub_image = self.sub_image(rect)?;
        imageops::clip_alpha(sub_image, min, max);
        Ok(())
    }

    #[inline]
    pub fn fill_rect(
        &mut self,
        color: impl Into<image::Rgba<u8>>,
        rect: &Rect,
        mode: imageops::FillMode,
    ) -> Result<(), FillError> {
        let size = self.size();
        let color = color.into();
        let sub_image = self.sub_image(rect).map_err(|err| FillError {
            size,
            rect: *rect,
            source: err,
        })?;
        imageops::fill_rect(sub_image, color, mode);
        Ok(())
    }

    #[inline]
    pub fn resize_and_crop(
        &mut self,
        size: impl Into<Size>,
        resize_mode: super::ResizeMode,
        crop_mode: super::CropMode,
    ) -> Result<(), ResizeAndCropError> {
        let size = size.into();
        self.resize(size, resize_mode)?;
        self.crop_to_fit(size, crop_mode)?;
        Ok(())
    }

    #[inline]
    pub fn fade_out(
        &mut self,
        start: impl Into<Point>,
        end: impl Into<Point>,
        axis: super::types::Axis,
    ) -> Result<(), FadeError> {
        use super::types::Axis;
        let start = start.into();
        let end = end.into();
        let switch_direction = match axis {
            Axis::X => start.x < end.x,
            Axis::Y => start.y < end.y,
        };

        match (|| {
            let sub_image_rect = Rect::from_points(start, end);
            let sub_image = self.sub_image(&sub_image_rect)?;
            imageops::fade_out(sub_image, axis, switch_direction)?;
            Ok::<_, FadeErrorSource>(())
        })() {
            Ok(_) => Ok(()),
            Err(err) => Err(FadeError {
                size: self.size(),
                start,
                end,
                axis,
                source: err,
            }),
        }
    }

    #[inline]
    pub fn resize(
        &mut self,
        size: impl Into<Size>,
        mode: super::ResizeMode,
    ) -> Result<(), ResizeError> {
        #[allow(unused_variables)]
        let start = Instant::now();

        let size = size.into();
        let resized = self
            .size()
            .scale_to(size, mode)
            .map_err(|err| ResizeError {
                size: self.size(),
                target: size,
                mode,
                source: err,
            })?;
        let filter = defaults::FILTER_TYPE;
        self.inner = imageops::resize(&self.inner, resized.width, resized.height, filter);

        debug!(
            "fitting to",
            resized,
            "took",
            start.elapsed_millis(),
            "msec"
        );
        Ok(())
    }

    #[inline]
    pub fn overlay(
        &mut self,
        overlay_image: &impl std::ops::Deref<Target = image::RgbaImage>,
        offset: impl Into<Point>,
    ) {
        // todo: should we check that offset + overlay_image is within bounds?
        let offset: Point = offset.into();
        imageops::overlay(&mut self.inner, &**overlay_image, offset.x, offset.y);
    }

    #[inline]
    pub fn crop_to_fit(
        &mut self,
        size: impl Into<Size>,
        mode: super::CropMode,
    ) -> Result<(), CropToFitError> {
        let target = size.into();
        match (|| {
            let rect = self.size().crop_to_fit(target, mode)?;
            self.crop(&rect).map_err(Box::new)?;
            Ok::<_, CropErrorSource>(())
        })() {
            Ok(_) => Ok(()),
            Err(err) => Err(CropToFitError {
                size: self.size(),
                target,
                source: err,
            }),
        }
    }

    #[inline]
    pub fn crop(&mut self, rect: &Rect) -> Result<(), CropRectError> {
        let size = self.size();
        self.inner = self
            .sub_image(rect)
            .map_err(|err| CropRectError {
                size,
                rect: *rect,
                source: err.into(),
            })?
            .to_image();
        Ok(())
    }

    #[inline]
    pub fn crop_sides(&mut self, sides: Sides) -> Result<(), CropSidesError> {
        match (|| {
            let rect = Rect::from(self.size()).checked_sub(sides)?;
            self.crop(&rect).map_err(Box::new)?;
            Ok::<_, CropErrorSource>(())
        })() {
            Ok(_) => Ok(()),
            Err(err) => Err(CropSidesError {
                sides,
                size: self.size(),
                source: err,
            }),
        }
    }

    #[inline]
    pub fn rotate(&mut self, angle: &super::Rotation) {
        use super::Rotation;
        if let Some(rotated) = match angle {
            Rotation::Rotate0 => None,
            Rotation::Rotate90 => Some(imageops::rotate90(&self.inner)),
            Rotation::Rotate180 => Some(imageops::rotate180(&self.inner)),
            Rotation::Rotate270 => Some(imageops::rotate270(&self.inner)),
        } {
            self.inner = rotated;
        }
    }

    #[inline]
    pub fn rotate_to_orientation(&mut self, orientation: super::Orientation) {
        if self.orientation() != orientation {
            self.rotate(&super::Rotation::Rotate90);
        }
    }

    #[inline]
    pub fn save_with_filename(
        &self,
        path: impl AsRef<Path>,
        quality: impl Into<Option<u8>>,
    ) -> Result<(), SaveError> {
        let path = path.as_ref();
        let quality = quality.into();
        match (|| {
            let format = ImageFormat::from_path(path)?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = fs::OpenOptions::new()
                .read(false)
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;
            self.encode_to(&mut file, format, quality)?;

            Ok::<_, SaveErrorSource>(())
        })() {
            Ok(_) => Ok(()),
            Err(err) => Err(SaveError {
                path: Some(path.to_path_buf()),
                format: ImageFormat::from_path(path).ok(),
                quality,
                source: err,
            }),
        }
    }

    #[inline]
    pub fn save(&self, quality: impl Into<Option<u8>>) -> Result<(), SaveError> {
        let quality = quality.into();
        let (default_output, _) = self.output_path(None);
        let path = default_output.ok_or(SaveError {
            path: None,
            format: None,
            quality,
            source: SaveErrorSource::MissingOutputPath,
        })?;
        self.save_with_filename(path, quality)
    }

    #[inline]
    pub fn encode_to(
        &self,
        w: &mut (impl std::io::Write + Seek),
        format: ImageFormat,
        quality: impl Into<Option<u8>>,
    ) -> Result<(), image::ImageError> {
        use image::{codecs, ImageEncoder, ImageOutputFormat};

        let data = self.inner.as_raw().as_ref();
        let color = image::ColorType::Rgba8;
        let quality = quality.into();
        let width = self.inner.width();
        let height = self.inner.height();
        match format.into() {
            ImageOutputFormat::Png => {
                codecs::png::PngEncoder::new(w).write_image(data, width, height, color)
            }
            ImageOutputFormat::Jpeg(_) => {
                let quality = quality.unwrap_or(defaults::JPEG_QUALITY);
                codecs::jpeg::JpegEncoder::new_with_quality(w, quality)
                    .write_image(data, width, height, color)
            }
            ImageOutputFormat::Gif => {
                codecs::gif::GifEncoder::new(w).encode(data, width, height, color)
            }
            ImageOutputFormat::Ico => {
                codecs::ico::IcoEncoder::new(w).write_image(data, width, height, color)
            }
            ImageOutputFormat::Bmp => {
                codecs::bmp::BmpEncoder::new(w).write_image(data, width, height, color)
            }
            ImageOutputFormat::Tiff => {
                codecs::tiff::TiffEncoder::new(w).write_image(data, width, height, color)
            }
            ImageOutputFormat::Unsupported(msg) => Err(image::error::ImageError::Unsupported(
                image::error::UnsupportedError::from_format_and_kind(
                    image::error::ImageFormatHint::Unknown,
                    image::error::UnsupportedErrorKind::Format(
                        image::error::ImageFormatHint::Name(msg),
                    ),
                ),
            )),
            _ => Err(image::error::ImageError::Unsupported(
                image::error::UnsupportedError::from_format_and_kind(
                    image::error::ImageFormatHint::Unknown,
                    image::error::UnsupportedErrorKind::Format(
                        image::error::ImageFormatHint::Name("missing format".to_string()),
                    ),
                ),
            )),
        }?;
        Ok(())
    }

    #[inline]
    fn output_path(&self, format: Option<ImageFormat>) -> (Option<PathBuf>, Option<ImageFormat>) {
        let source_format = self
            .path
            .as_ref()
            .and_then(|p| ImageFormat::from_path(p).ok());
        let format = format.or(source_format);
        let ext = format
            .unwrap_or(ImageFormat::Jpeg)
            .extensions_str()
            .iter()
            .next()
            .unwrap_or(&"jpg");
        let path = self.path.as_ref().and_then(|p| {
            p.file_stem()
                .map(|stem| format!("{}_with_border.{}", &stem.to_string_lossy(), &ext))
                .map(|filename| p.with_file_name(filename))
        });
        (path, format)
    }
}

#[cfg(test)]
mod tests {
    use super::{Image, ImageFormat};
    use image::RgbaImage;

    macro_rules! output_path_tests {
        ($($name:ident: $values:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (path, format, want_path, want_format): (
                        Option<&str>,
                        Option<ImageFormat>,
                        Option<&str>,
                        Option<ImageFormat>
                    ) = $values;
                    let img = Image {
                        inner: RgbaImage::new(32, 32),
                        path: path.map(Into::into),
                    };
                    let (have_path, have_format) = img.output_path(format);
                    assert_eq!(have_path, want_path.map(Into::into));
                    assert_eq!(have_format, want_format);
                    if let Some(p) = have_path {
                        assert_eq!(
                            ImageFormat::from_path(p).ok(),
                            want_format
                        );
                    };
                }
            )*
        }
    }

    output_path_tests! {
        test_no_path_no_format: (None, None, None, None),
        test_jpg_path_no_format: (
           Some("samples/lowres.jpg"), None,
           Some("samples/lowres_with_border.jpg"), Some(ImageFormat::Jpeg)
        ),
        test_png_path_no_format: (
           Some("samples/lowres.png"), None,
           Some("samples/lowres_with_border.png"), Some(ImageFormat::Png)
        ),
        test_no_path_jpg_format: (
           None, Some(ImageFormat::Jpeg),
           None, Some(ImageFormat::Jpeg)
        ),
        test_no_path_png_format: (
           None, Some(ImageFormat::Png),
           None, Some(ImageFormat::Png)
        ),
        test_jpg_path_jpg_format: (
           Some("samples/lowres.jpg"), Some(ImageFormat::Jpeg),
           Some("samples/lowres_with_border.jpg"), Some(ImageFormat::Jpeg)
        ),
        test_jpg_path_png_format: (
           Some("samples/lowres.jpg"), Some(ImageFormat::Png),
           Some("samples/lowres_with_border.png"), Some(ImageFormat::Png)
        ),
        test_png_path_jpg_format: (
           Some("samples/lowres.png"), Some(ImageFormat::Jpeg),
           Some("samples/lowres_with_border.jpg"), Some(ImageFormat::Jpeg)
        ),
        test_png_path_png_format: (
           Some("samples/lowres.png"), Some(ImageFormat::Png),
           Some("samples/lowres_with_border.png"), Some(ImageFormat::Png)
        ),
    }
}
