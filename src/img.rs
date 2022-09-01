use crate::defaults;
use crate::types::{Point, Size};
use crate::utils;
use crate::Error;
use image::{
    codecs, io::Reader as ImageReader, ColorType, DynamicImage, ImageEncoder, ImageFormat,
    ImageOutputFormat, Pixel, Rgba, RgbaImage,
};

use std::cmp::{max, min};
use std::fs;
use std::io::{BufReader, Seek};
use std::path::{Path, PathBuf};

#[inline]
pub fn fill_rect(image: &mut RgbaImage, color: &Rgba<u8>, top_left: Point, bottom_right: Point) {
    let x1 = utils::clamp(min(top_left.x, bottom_right.x), 0, image.width());
    let x2 = utils::clamp(max(top_left.x, bottom_right.x), 0, image.width());
    let y1 = utils::clamp(min(top_left.y, bottom_right.y), 0, image.height());
    let y2 = utils::clamp(max(top_left.y, bottom_right.y), 0, image.height());
    for x in x1..x2 {
        for y in y1..y2 {
            image.get_pixel_mut(x, y).blend(color);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[inline]
pub fn fade_out(image: &mut RgbaImage, start: u32, end: u32, direction: Direction) {
    let other = match direction {
        Direction::Horizontal => image.height(),
        Direction::Vertical => image.width(),
    };
    let diff = (end as f32 - start as f32).abs();
    for i in min(start, end)..=max(start, end) {
        let ir = i - min(start, end);
        let mut frac = ir as f32 / diff;
        if start < end {
            frac = 1.0 - frac;
        }
        let alpha = (255.0 * frac) as u8;
        for j in 0..other {
            let (x, y) = match direction {
                Direction::Horizontal => (i, j),
                Direction::Vertical => (j, i),
            };
            let channels = image.get_pixel_mut(x, y).channels_mut();
            channels[3] = min(channels[3], alpha);
        }
    }
}

#[derive(Debug)]
pub struct Image {
    pub(crate) inner: RgbaImage,
    pub(crate) path: Option<PathBuf>,
    pub(crate) size: Size,
}

impl Image {
    pub fn data(&self) -> RgbaImage {
        self.inner.clone()
    }

    pub fn as_raw(&self) -> &[u8] {
        self.inner.as_raw()
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn from_image(image: DynamicImage) -> Self {
        let inner = image.to_rgba8();
        let size = Size {
            width: inner.width(),
            height: inner.height(),
        };
        Self {
            inner,
            size,
            path: None,
        }
    }

    pub fn new<R: std::io::BufRead + std::io::Seek>(reader: R) -> Result<Self, Error> {
        let reader = ImageReader::new(reader).with_guessed_format()?;
        let inner = reader.decode()?.to_rgba8();
        let size = Size {
            width: inner.width(),
            height: inner.height(),
        };
        Ok(Self {
            inner,
            path: None,
            size,
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = fs::OpenOptions::new().read(true).open(&path)?;
        let mut img = Self::new(BufReader::new(&file))?;
        img.path = Some(path.as_ref().to_path_buf());
        Ok(img)
    }

    pub fn save<P: AsRef<Path>>(&self, path: Option<P>, quality: Option<u8>) -> Result<(), Error> {
        let (default_output, _) = self.output_path(None);
        let path = path
            .as_ref()
            .map(|p| p.as_ref())
            .or(default_output.as_ref().map(|p| p.as_path()))
            .ok_or(Error::MissingOutputFile)?;

        let format = ImageFormat::from_path(&path)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        self.encode_to(&mut file, format, quality)
    }

    pub fn encode_to<W: std::io::Write + Seek>(
        &self,
        w: &mut W,
        format: ImageFormat,
        quality: Option<u8>,
    ) -> Result<(), Error> {
        let data = self.inner.as_raw().as_ref();
        let color = ColorType::Rgba8;
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

    fn output_path(&self, format: Option<ImageFormat>) -> (Option<PathBuf>, Option<ImageFormat>) {
        let source_format = self
            .path
            .as_ref()
            .and_then(|p| ImageFormat::from_path(p).ok());
        let format = format.or(source_format); // .unwrap_or(ImageFormat::Jpeg);
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
        // .map(|p| (p, format))
    }
}

#[cfg(test)]
mod tests {
    use super::Image;
    use crate::{ImageFormat, Size};
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
                        size: Size::default(),
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
