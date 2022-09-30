use super::defaults;
pub use super::imageops::*;
use super::numeric::ops::CheckedSub;
use super::sides::{abs::Sides, percent::Sides as SidesPercent};
use super::types::*;
use super::Error;
use image::{
    codecs, io::Reader as ImageReader, ColorType, DynamicImage, ImageEncoder, ImageFormat,
    ImageOutputFormat, RgbaImage,
};

use std::cmp::max;
use std::fs;
use std::io::{BufReader, Seek};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Image {
    pub(crate) inner: RgbaImage,
    pub(crate) path: Option<PathBuf>,
}

impl AsRef<image::RgbaImage> for Image {
    fn as_ref(&self) -> &image::RgbaImage {
        &self.inner
    }
}

impl Image {
    pub fn data(&self) -> RgbaImage {
        self.inner.clone()
    }

    pub fn as_raw(&self) -> &[u8] {
        self.inner.as_raw()
    }

    pub fn size(&self) -> Size {
        Size::from(&self.inner)
    }

    pub fn new(width: u32, height: u32) -> Self {
        let inner = RgbaImage::new(width, height);
        Self { inner, path: None }
    }

    pub fn with_size<S: Into<Size>>(size: S) -> Self {
        let size = size.into();
        Self::new(size.width, size.height)
    }

    pub fn from_image(image: DynamicImage) -> Self {
        let inner = image.to_rgba8();
        Self { inner, path: None }
    }

    pub fn from_reader<R: std::io::BufRead + std::io::Seek>(reader: R) -> Result<Self, Error> {
        let reader = ImageReader::new(reader).with_guessed_format()?;
        let inner = reader.decode()?.to_rgba8();
        Ok(Self { inner, path: None })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = fs::OpenOptions::new().read(true).open(&path)?;
        let mut img = Self::from_reader(BufReader::new(&file))?;
        img.path = Some(path.as_ref().to_path_buf());
        Ok(img)
    }

    pub fn is_portrait(&self) -> bool {
        self.size().is_portrait()
    }

    pub fn orientation(&self) -> Orientation {
        self.size().orientation()
    }

    pub fn fill<C: Into<image::Rgba<u8>>>(&mut self, color: C, mode: FillMode) {
        let size = self.size();
        self.fill_rect(color.into(), Point::origin(), size, mode);
    }

    pub fn fill_rect<TL, S, C>(&mut self, color: C, top_left: TL, size: S, mode: FillMode)
    where
        TL: Into<Point>,
        S: Into<Size>,
        C: Into<image::Rgba<u8>>,
    {
        fill_rect(self, color.into(), top_left.into(), size.into(), mode);
    }

    pub fn resize_to_fit<S>(&mut self, container: S, resize_mode: ResizeMode, crop_mode: CropMode)
    where
        S: Into<Size>,
    {
        let container = container.into();
        self.resize(container, resize_mode);
        self.crop_to_fit(container, crop_mode);
    }

    #[inline]
    pub fn fade_out<P1, P2>(&mut self, top_left: P1, bottom_right: P2, axis: Axis)
    where
        P1: Into<Point>,
        P2: Into<Point>,
    {
        fade_out(self, top_left.into(), bottom_right.into(), axis);
    }

    pub fn resize<S>(&mut self, size: S, mode: ResizeMode)
    where
        S: Into<Size>,
    {
        #[cfg(debug_assertions)]
        let start = chrono::Utc::now().time();

        let size = self.size().scale_to(size.into(), mode);
        self.inner = resize(&self.inner, size.width, size.height, defaults::FILTER_TYPE);

        #[cfg(debug_assertions)]
        crate::debug!(
            "fitting to {} took {:?} msec",
            size,
            (chrono::Utc::now().time() - start).num_milliseconds(),
        );
    }

    pub fn crop_to_fit<S>(&mut self, container: S, mode: CropMode)
    where
        S: Into<Size>,
    {
        let container = container.into();
        let crop = self.size().crop_to_fit(container, mode);
        self.crop_sides(crop);
    }

    pub fn overlay<O, P>(&mut self, overlay_image: &O, offset: P)
    where
        O: image::GenericImageView<Pixel = image::Rgba<u8>>,
        P: Into<Point>,
    {
        let offset: Point = offset.into();
        overlay(&mut self.inner, overlay_image, offset.x, offset.y);
    }

    pub fn crop(&mut self, top_left: Point, bottom_right: Point) {
        let cropped_size = Size::try_from(bottom_right.checked_sub(top_left).unwrap()).unwrap();
        self.inner = crop(
            &mut self.inner,
            max(0, top_left.x) as u32,
            max(0, top_left.y) as u32,
            cropped_size.width,
            cropped_size.height,
        )
        .to_image();
    }

    pub fn crop_sides(&mut self, crop_sides: Sides) {
        let cropped_size = self.size().checked_sub(crop_sides).unwrap();
        self.inner = crop(
            &mut self.inner,
            crop_sides.left,
            crop_sides.top,
            cropped_size.width,
            cropped_size.height,
        )
        .to_image();
    }

    pub fn rotate(&mut self, angle: &Rotation) {
        if let Some(rotated) = match angle {
            Rotation::Rotate0 => None,
            Rotation::Rotate90 => Some(rotate90(&self.inner)),
            Rotation::Rotate180 => Some(rotate180(&self.inner)),
            Rotation::Rotate270 => Some(rotate270(&self.inner)),
        } {
            self.inner = rotated;
        }
    }

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

    pub fn save(&self, quality: impl Into<Option<u8>>) -> Result<(), Error> {
        let (default_output, _) = self.output_path(None);
        let path = default_output.ok_or(Error::MissingOutputFile)?;
        self.save_with_filename(path, quality)
    }

    pub fn encode_to(
        &self,
        w: &mut (impl std::io::Write + Seek),
        format: ImageFormat,
        quality: impl Into<Option<u8>>,
    ) -> Result<(), Error> {
        let data = self.inner.as_raw().as_ref();
        let color = ColorType::Rgba8;
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
    use super::Image;
    use crate::ImageFormat;
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
