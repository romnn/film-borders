use crate::types::{Point, Size};
use crate::utils;
use crate::Error;
use image::{
    codecs, imageops, io::Reader as ImageReader, ColorType, DynamicImage, ImageEncoder, ImageError,
    ImageFormat, ImageOutputFormat, Pixel, Rgba, RgbaImage,
};

use std::cmp::{max, min};
use std::env;
use std::fs;
use std::io::{BufReader, Seek};
use std::io::{Error as IOError, ErrorKind};
use std::path::{Path, PathBuf};

const DEFAULT_JPEG_QUALITY: u8 = 70; // 1-100

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
        // println!("alpha = {} = {} / {}", alpha, ir, range);
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
    inner: RgbaImage,
    pub path: Option<PathBuf>,
    size: Size,
}

impl Image {
    pub fn data(&self) -> RgbaImage {
        self.inner.clone()
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
        let default_output = self.output_path(None);
        let path = path
            .as_ref()
            .map(|p| p.as_ref())
            .or(default_output.as_ref().map(|p| p.as_path()))
            .ok_or(Error::MissingOutputFile)?;

        let format = ImageFormat::from_path(&path)?;
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
                let quality = quality.unwrap_or(DEFAULT_JPEG_QUALITY);
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

    fn output_path(&self, format: Option<ImageFormat>) -> Option<PathBuf> {
        let source_format = self
            .path
            .as_ref()
            .and_then(|p| ImageFormat::from_path(p).ok());
        let format = format.or(source_format).unwrap_or(ImageFormat::Jpeg);
        let ext = format.extensions_str().iter().next().unwrap_or(&"jpg");
        self.path.as_ref().and_then(|p| {
            p.file_stem()
                .map(|stem| format!("{}_with_border.{}", &stem.to_string_lossy(), &ext))
                .map(|filename| p.with_file_name(filename))
        })
    }
}
