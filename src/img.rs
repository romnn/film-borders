use super::arithmetic::{ops::CheckedSub, Cast, CastError};
use super::types::{sides::abs::Sides, Point, Rect, Size};
use super::{defaults, error, imageops};
pub use image::ImageFormat;
use std::borrow::Borrow;
use std::fs;
use std::io::{BufReader, Seek};
use std::path::{Path, PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("point {point} exceeds image bounds {bounds}")]
    OutOfBounds { point: Point, bounds: Rect },

    #[error("missing output file path")]
    MissingOutputPath,

    #[error(transparent)]
    Arithmetic(#[from] error::Arithmetic),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::error::ImageError),
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
    pub fn with_size<S: Into<Size>>(size: S) -> Self {
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
    pub fn from_reader<R: std::io::BufRead + std::io::Seek>(reader: R) -> Result<Self, Error> {
        let reader = image::io::Reader::new(reader).with_guessed_format()?;
        let inner = reader.decode()?.to_rgba8();
        Ok(Self { inner, path: None })
    }

    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = fs::OpenOptions::new().read(true).open(&path)?;
        let mut img = Self::from_reader(BufReader::new(&file))?;
        img.path = Some(path.as_ref().to_path_buf());
        Ok(img)
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
    pub fn fill<C: Into<image::Rgba<u8>>>(&mut self, color: C, mode: imageops::FillMode) {
        let rect: Rect = self.size().into();
        self.fill_rect(color.into(), &rect, mode);
    }

    #[inline]
    pub fn fill_rect(
        &mut self,
        color: impl Into<image::Rgba<u8>>,
        rect: &Rect,
        mode: imageops::FillMode,
    ) -> Result<(), Error> {
        let color = color.into();
        let rect = self.subimage_rect(rect)?;
        imageops::fill_rect(self, color, &rect, mode)
    }

    #[inline]
    pub fn resize_to_fit(
        &mut self,
        container: impl Into<Size>,
        resize_mode: super::ResizeMode,
        crop_mode: super::CropMode,
    ) {
        let container = container.into();
        self.resize(container, resize_mode);
        self.crop_to_fit(container, crop_mode);
    }

    #[inline]
    pub fn fade_out(
        &mut self,
        start: impl Into<Point>,
        end: impl Into<Point>,
        axis: super::types::Axis,
    ) -> Result<(), Error> {
        imageops::fade_out(self, start.into(), end.into(), axis)
    }

    #[inline]
    pub fn resize(&mut self, size: impl Into<Size>, mode: super::ResizeMode) {
        #[cfg(debug_assertions)]
        let start = chrono::Utc::now().time();

        let size = self.size().scale_to(size.into(), mode);
        self.inner = imageops::resize(&self.inner, size.width, size.height, defaults::FILTER_TYPE);

        #[cfg(debug_assertions)]
        crate::debug!(
            "fitting to {} took {:?} msec",
            size,
            (chrono::Utc::now().time() - start).num_milliseconds(),
        );
    }

    #[inline]
    pub fn crop_to_fit<S>(&mut self, container: S, mode: super::CropMode)
    where
        S: Into<Size>,
    {
        let container = container.into();
        let crop = self.size().crop_to_fit(container, mode);
        self.crop_sides(crop);
    }

    #[inline]
    pub fn overlay(
        &mut self,
        overlay_image: &impl std::ops::Deref<Target = image::RgbaImage>,
        offset: impl Into<Point>,
    ) {
        let offset: Point = offset.into();
        imageops::overlay(&mut self.inner, &**overlay_image, offset.x, offset.y);
    }

    #[inline]
    pub fn crop(&mut self, top_left: Point, bottom_right: Point) {
        let cropped_size = Size::try_from(bottom_right.checked_sub(top_left).unwrap()).unwrap();
        let top_left: Size = Size::try_from(top_left).unwrap();
        self.inner = imageops::crop(
            &mut self.inner,
            top_left.width,
            top_left.height,
            cropped_size.width,
            cropped_size.height,
        )
        .to_image();
    }

    #[inline]
    pub fn crop_sides(&mut self, crop_sides: Sides) {
        let cropped_size = self.size().checked_sub(crop_sides).unwrap();
        self.inner = imageops::crop(
            &mut self.inner,
            crop_sides.left,
            crop_sides.top,
            cropped_size.width,
            cropped_size.height,
        )
        .to_image();
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
    ) -> Result<(), Error> {
        let path = path.as_ref();
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
        self.encode_to(&mut file, format, quality)
    }

    #[inline]
    pub fn save(&self, quality: impl Into<Option<u8>>) -> Result<(), Error> {
        let (default_output, _) = self.output_path(None);
        let path = default_output.ok_or(Error::MissingOutputPath)?;
        self.save_with_filename(path, quality)
    }

    #[inline]
    pub fn encode_to(
        &self,
        w: &mut (impl std::io::Write + Seek),
        format: ImageFormat,
        quality: impl Into<Option<u8>>,
    ) -> Result<(), Error> {
        use image::{codecs, ImageEncoder, ImageOutputFormat};

        let data = self.inner.as_raw().as_ref();
        let color = image::ColorType::Rgba8;
        let quality = quality.into();
        let width = self.inner.width();
        let height = self.inner.height();
        match format.into() {
            ImageOutputFormat::Png => codecs::png::PngEncoder::new(w)
                .write_image(data, width, height, color)
                .map_err(Error::from),
            ImageOutputFormat::Jpeg(_) => {
                let quality = quality.unwrap_or(defaults::JPEG_QUALITY);
                codecs::jpeg::JpegEncoder::new_with_quality(w, quality)
                    .write_image(data, width, height, color)
                    .map_err(Error::from)
            }
            ImageOutputFormat::Gif => codecs::gif::GifEncoder::new(w)
                .encode(data, width, height, color)
                .map_err(Error::from),
            ImageOutputFormat::Ico => codecs::ico::IcoEncoder::new(w)
                .write_image(data, width, height, color)
                .map_err(Error::from),
            ImageOutputFormat::Bmp => codecs::bmp::BmpEncoder::new(w)
                .write_image(data, width, height, color)
                .map_err(Error::from),
            ImageOutputFormat::Tiff => codecs::tiff::TiffEncoder::new(w)
                .write_image(data, width, height, color)
                .map_err(Error::from),
            ImageOutputFormat::Unsupported(msg) => {
                Err(Error::from(image::error::ImageError::Unsupported(
                    image::error::UnsupportedError::from_format_and_kind(
                        image::error::ImageFormatHint::Unknown,
                        image::error::UnsupportedErrorKind::Format(
                            image::error::ImageFormatHint::Name(msg),
                        ),
                    ),
                )))
            }
            _ => Err(Error::from(image::error::ImageError::Unsupported(
                image::error::UnsupportedError::from_format_and_kind(
                    image::error::ImageFormatHint::Unknown,
                    image::error::UnsupportedErrorKind::Format(
                        image::error::ImageFormatHint::Name("missing format".to_string()),
                    ),
                ),
            ))),
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

mod sealed {
    use super::{Error, Image};
    use crate::arithmetic::{Cast, CastError};
    use crate::error;
    use crate::types::Rect;

    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    #[non_exhaustive]
    pub struct ImageRect {
        pub top: u32,
        pub left: u32,
        pub bottom: u32,
        pub right: u32,
        _sealed: (),
    }

    impl Image {
        #[inline]
        pub fn subimage_rect(&mut self, rect: &Rect) -> Result<ImageRect, Error> {
            let image_rect: Rect = self.size().into();

            if !image_rect.contains(&rect.top_left()) {
                return Err(Error::OutOfBounds {
                    bounds: image_rect,
                    point: rect.top_left(),
                });
            }

            if !image_rect.contains(&rect.bottom_right()) {
                return Err(Error::OutOfBounds {
                    bounds: image_rect,
                    point: rect.bottom_right(),
                });
            }

            match (|| {
                Ok::<_, CastError<i64, u32>>(ImageRect {
                    top: rect.top.cast::<u32>()?,
                    left: rect.left.cast::<u32>()?,
                    bottom: rect.bottom.cast::<u32>()?,
                    right: rect.right.cast::<u32>()?,
                    _sealed: (),
                })
            })() {
                Ok(rect) => Ok(rect),
                Err(err) => Err(Error::Arithmetic(error::Arithmetic {
                    msg: format!(
                        "failed to get subview {} into image of size {}",
                        rect,
                        self.size()
                    ),
                    source: err.into(),
                })),
            }
        }
    }
}

pub use sealed::ImageRect;

impl ImageRect {
    // pub fn size(&self) -> Result<Size, error::Arithmetic> {
    #[inline]
    #[must_use]
    pub fn size(&self) -> Size {
        // safety: this is safe because these invariants hold:
        // 1. top <= bottom
        // 2. left <=  right
        Size {
            width: self.right - self.left,
            height: self.bottom - self.top,
        }

        // match (|| {
        //     Ok::<Size, ops::SubError<u32, u32>>(Size {
        //         width: self.right.checked_sub(self.left)?,
        //         height: self.bottom.checked_sub(self.top)?,
        //     })
        // })() {
        //     Ok(size) => Ok(size),
        //     Err(err) => Err(error::Arithmetic {
        //         msg: format!("failed to compute size for {}", self),
        //         source: err.into(),
        //     }),
        // }
    }

    #[inline]
    pub fn x_coords(&self) -> std::ops::Range<u32> {
        self.left..self.right
    }

    #[inline]
    pub fn y_coords(&self) -> std::ops::Range<u32> {
        self.top..self.bottom
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
