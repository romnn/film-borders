#[cfg(feature = "borders")]
use super::borders;
use super::error::{BorderError, ColorError, ParseEnumError};
use super::utils::{Ceil, Floor, Round, RoundingMode};
use super::{imageops, img, utils, Error};
use cgmath::Rotation as VecRotation;
use num::traits::{Float, NumCast};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::path::Path;
use wasm_bindgen::prelude::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Rect {
    pub top: i64,
    pub left: i64,
    pub right: i64,
    pub bottom: i64,
}

impl Rect {
    pub fn new(top_left: Point, size: Size) -> Self {
        let bottom_right: Point = top_left + Point::from(size);
        Self {
            top: top_left.y,
            left: top_left.x,
            bottom: bottom_right.y,
            right: bottom_right.x,
        }
    }

    #[inline]
    pub fn pixel_count(&self) -> u64 {
        self.width() as u64 * self.height() as u64
    }

    #[inline]
    pub fn size(&self) -> Size {
        let size = self.bottom_right() - self.top_left();
        size.into()
    }

    #[inline]
    pub fn center(&self) -> Point {
        self.top_left() + (self.size() / 2.0f64)
    }

    #[inline]
    pub fn crop_mode(&self, container: &Rect) -> CropMode {
        // let offset = self.center() - container.center();
        let offset = container.center() - self.center();
        // offset.unit_vector()
        CropMode::Custom {
            x: offset.x,
            y: offset.y,
        }
        // let top_left =
        // let center = self.top_left();
        // let contaienr_center = container.top_left()
        // self.top_left() + (self.size() / 2.0f64)
    }

    #[inline]
    pub fn top_left(&self) -> Point {
        Point {
            y: self.top,
            x: self.left,
        }
    }

    #[inline]
    pub fn bottom_right(&self) -> Point {
        Point {
            y: self.bottom,
            x: self.right,
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.size().width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.size().height
    }

    #[inline]
    pub fn intersects(&self, other: &Self, padding: i64) -> bool {
        let top_left = self.contains(other.left, other.top, padding);
        let bottom_right = self.contains(other.right, other.bottom, padding);
        top_left || bottom_right
    }

    #[inline]
    pub fn extend_to(&mut self, point: Point) {
        self.top = min(self.top, point.y);
        self.left = min(self.left, point.x);
        self.bottom = max(self.bottom, point.y);
        self.right = max(self.right, point.x);
    }

    #[inline]
    pub fn extend(self, value: u32) -> Self {
        self + Sides::uniform(value)
    }

    #[inline]
    pub fn contains(&self, x: i64, y: i64, padding: i64) -> bool {
        let x_left = self.left - padding;
        let x_right = self.right + padding;
        let y_top = self.top - padding;
        let y_bottom = self.bottom + padding;

        x_left <= x && x <= x_right && y_top <= y && y <= y_bottom
    }

    #[inline]
    pub fn clip_to(self, bounds: &Self) -> Self {
        let top = utils::clamp(self.top, bounds.top, bounds.bottom);
        let bottom = utils::clamp(self.bottom, bounds.top, bounds.bottom);
        let left = utils::clamp(self.left, bounds.left, bounds.right);
        let right = utils::clamp(self.right, bounds.left, bounds.right);
        Self {
            top,
            bottom,
            left,
            right,
        }
    }
}

impl From<Size> for Rect {
    fn from(size: Size) -> Self {
        Self {
            top: 0,
            bottom: size.height as i64,
            left: 0,
            right: size.width as i64,
        }
    }
}

impl From<Sides> for Rect {
    fn from(sides: Sides) -> Self {
        Self {
            top: sides.top as i64,
            bottom: sides.bottom as i64,
            left: sides.left as i64,
            right: sides.right as i64,
        }
    }
}

impl std::ops::Add<Point> for Rect {
    type Output = Self;

    fn add(self, point: Point) -> Self::Output {
        Self {
            top: self.top + point.y,
            left: self.left + point.x,
            bottom: self.bottom + point.y,
            right: self.right + point.x,
        }
    }
}

// impl std::ops::Add for Rect {
//     type Output = Self;

//     fn add(self, other: Self) -> Self::Output {
//         Self {
//             top: self.top + other.top,
//             left: self.left + other.left,
//             bottom: self.bottom + other.bottom,
//             right: self.right + other.right,
//         }
//     }
// }

// impl std::ops::Sub for Rect {
//     type Output = Self;

//     fn sub(self, other: Self) -> Self::Output {
//         Self {
//             top: self.top - other.top,
//             left: self.left - other.left,
//             bottom: self.bottom - other.bottom,
//             right: self.right - other.right,
//         }
//     }
// }

impl std::ops::Sub<Sides> for Rect {
    type Output = Self;

    fn sub(self, sides: Sides) -> Self::Output {
        Self {
            top: self.top + sides.top as i64,
            left: self.left + sides.left as i64,
            bottom: self.bottom - sides.bottom as i64,
            right: self.right - sides.right as i64,
        }
    }
}

impl std::ops::Add<Sides> for Rect {
    type Output = Self;

    fn add(self, sides: Sides) -> Self::Output {
        Self {
            top: self.top - sides.top as i64,
            left: self.left - sides.left as i64,
            bottom: self.bottom + sides.bottom as i64,
            right: self.right + sides.right as i64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BorderOptions {
    pub transparent_component_threshold: i64,
    pub alpha_threshold: f32,
}

impl Default for BorderOptions {
    #[inline]
    fn default() -> Self {
        Self {
            transparent_component_threshold: 8,
            alpha_threshold: 0.95,
        }
    }
}

#[derive(Clone)]
pub struct Border {
    pub inner: img::Image,
    options: Option<BorderOptions>,
    transparent_components: Vec<Rect>,
}

impl AsRef<image::RgbaImage> for Border {
    fn as_ref(&self) -> &image::RgbaImage {
        self.inner.as_ref()
    }
}

impl Border {
    #[inline]
    pub fn from_reader<R: std::io::BufRead + std::io::Seek>(
        reader: R,
        options: Option<BorderOptions>,
    ) -> Result<Self, Error> {
        Self::from_image(img::Image::from_reader(reader)?, options)
    }

    #[inline]
    pub fn open<P: AsRef<Path>>(path: P, options: Option<BorderOptions>) -> Result<Self, Error> {
        Self::from_image(img::Image::open(path)?, options)
    }

    #[inline]
    fn compute_transparent_components(
        &mut self,
        options: Option<BorderOptions>,
    ) -> Result<(), BorderError> {
        let options = options.unwrap_or(Default::default());
        self.transparent_components = imageops::find_transparent_components(
            &self.inner,
            options.alpha_threshold,
            options.transparent_component_threshold,
        );

        if self.transparent_components.is_empty() {
            return Err(BorderError::BadTransparency(
                self.transparent_components.clone(),
            ));
        }
        self.transparent_components
            .sort_by(|a, b| b.pixel_count().partial_cmp(&a.pixel_count()).unwrap());
        Ok(())
    }

    #[inline]
    pub fn from_image(inner: img::Image, options: Option<BorderOptions>) -> Result<Self, Error> {
        let mut border = Self {
            inner,
            options,
            transparent_components: Vec::new(),
        };
        border.compute_transparent_components(options)?;
        Ok(border)
    }

    #[inline]
    pub fn resize_to_fit(&mut self, container: Size, resize_mode: ResizeMode) -> Result<(), Error> {
        // let padding = Sides::default();
        self.inner
            .resize_to_fit(container, resize_mode, CropMode::Center);
        self.compute_transparent_components(self.options)?;
        Ok(())
        // let size = self.size().scale_to(container, resize_mode);
        // self.inner = imageops::resize(&self.inner, size.width, size.height, defaults::FILTER_TYPE);
        // if self.inner.orientation() != orientation {
        //     self.inner.rotate(Rotation::Rotate90);
        //     // rotate the transparent components by 90 degree to the right
        //     for c in self.transparent_components.iter_mut() {
        //         todo!();
        //     }
        // }
    }

    #[inline]
    pub fn rotate(&mut self, angle: Rotation) -> Result<(), Error> {
        self.inner.rotate(angle);
        self.compute_transparent_components(self.options)?;
        Ok(())

        // todo: rotate the transparent components
        // for c in self.transparent_components.iter_mut() {
        //     let top_left = Point {
        //         y: c.top,
        //         x: c.left,
        //     };
        //     let bottom_right = Point {
        //         y: c.bottom,
        //         x: c.right,
        //     };
        //     crate::debug!(&top_left);
        //     crate::debug!(&bottom_right);

        //     let top_left = top_left.rotate(angle);
        //     let bottom_right = bottom_right.rotate(angle);
        //     crate::debug!(&top_left);
        //     crate::debug!(&bottom_right);

        //     c.top = top_left.y;
        //     c.left = top_left.x;
        //     c.bottom = bottom_right.y;
        //     c.right = bottom_right.x;

        //     if angle == Rotation::Rotate90 {
        //         todo!();
        //     }
        // }
    }

    #[inline]
    pub fn rotate_to_orientation(&mut self, orientation: img::Orientation) -> Result<(), Error> {
        if self.inner.orientation() != orientation {
            self.rotate(Rotation::Rotate90)?;
        }
        Ok(())
    }

    #[inline]
    pub fn content_size(&self) -> Size {
        self.transparent_components.first().unwrap().size()
    }

    #[inline]
    pub fn size_for(&self, content: Size) -> Size {
        let content_size = self.content_size();
        let border_size = self.inner.size();
        // crate::debug!(&self.inner.size());
        // crate::debug!(&content_size);
        // crate::debug!(&content);

        let scale = content_size.scale_factor(content, ResizeMode::Cover);
        crate::debug!(
            // "scale {} image by {} to fit {} border content size {}",
            "scale {} border with content size {} to fit {} image (scale factor {})",
            border_size,
            content_size,
            content,
            &scale.0,
        );
        self.size().scale_by::<_, Round>(scale.0)
    }

    #[inline]
    pub fn transparent_components(&self) -> &Vec<Rect> {
        &self.transparent_components
    }

    #[inline]
    pub fn size(&self) -> Size {
        self.inner.size()
    }
}

#[derive()]
pub enum BorderSource {
    #[cfg(feature = "borders")]
    Builtin(borders::BuiltinBorder),
    Custom(Border),
}

impl BorderSource {
    #[inline]
    pub fn into_border(self) -> Result<Border, Error> {
        match self {
            #[cfg(feature = "borders")]
            Self::Builtin(builtin) => builtin.into_border(),
            Self::Custom(border) => Ok(border),
        }
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Mode {
    FitImage,
    ScaleBorder,
}

impl Default for Mode {
    #[inline]
    fn default() -> Self {
        Mode::FitImage
    }
}

impl std::str::FromStr for Mode {
    type Err = ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "fit" => Ok(Mode::FitImage),
            "scale" => Ok(Mode::ScaleBorder),
            _ => Err(ParseEnumError::Unknown(s.to_string())),
        }
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Copy, Clone)]
pub struct OutputSize {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl From<Size> for OutputSize {
    fn from(size: Size) -> Self {
        Self {
            width: Some(size.width),
            height: Some(size.height),
        }
    }
}

impl OutputSize {
    pub fn min(self, other: Self) -> Self {
        let width = utils::opt_min(self.width, other.width);
        let height = utils::opt_min(self.height, other.height);
        Self { width, height }
    }

    // pub fn scale_to(self, bounds: Self, mode: ResizeMode) -> Self {
    //     // no bounds: fine
    //     // w,h bounds: fine
    //     // w bounds: should
    //     // let bounds = self.min(container);
    //     // match options.output_size.min(options.output_size_bounds) {
    //     match bounds {
    //         // if no bounds
    //         OutputSize {
    //             width: None,
    //             height: None,
    //         } => self,
    //         // if some dimension is bounded,
    //         // scale by default content aspect ratio
    //         // or no just clamp
    //         OutputSize {
    //             width: None,
    //             height: Some(height),
    //         } => {
    //             // height changed, compute ratio
    //             let ratio = height as f64 / output_size.height as f64;
    //             let width = output_size.width as f64 * ratio;
    //             Self {
    //                 width: width as u32,
    //                 height,
    //             }
    //         }
    //         OutputSize {
    //             width: Some(width),
    //             height: None,
    //         } => {
    //             let ratio = width as f64 / output_size.width as f64;
    //             let height = output_size.height as f64 * ratio;
    //             Self {
    //                 width,
    //                 height: height as u32,
    //             }
    //         }
    //         // if only absolute values, nothing to be done
    //         OutputSize {
    //             width: Some(width),
    //             height: Some(height),
    //         } => Self { width, height },
    //     }

    //     // Self {
    //     //     width,
    //     //     height,
    //     // }
    //     // let container = container.into();
    //     match mode {
    //         ResizeMode::Fill => container,
    //         _ => {
    //             let scale = self.scale_factor(container, mode);
    //             self.scale_by::<_, Ceil>(scale.0)
    //         }
    //     }
    // }
}

#[wasm_bindgen]
impl OutputSize {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        OutputSize::default()
    }
}

macro_rules! from_hex {
    ($value:expr) => {
        $value.ok_or(ColorError::MissingComponent).and_then(|v| {
            u8::from_str_radix(v.as_str(), 16)
                .map_err(|_| ColorError::InvalidHex(v.as_str().to_owned()))
        })
    };
}

#[inline]
fn hex_to_color(hex: &str) -> Result<Color, ColorError> {
    lazy_static::lazy_static! {
        pub static ref HEX_REGEX: Regex = Regex::new(r"^[\s#]*(?P<r>[a-f\d]{2})(?P<g>[a-f\d]{2})(?P<b>[a-f\d]{2})\s*$").unwrap();
    };
    let hex = hex.to_ascii_lowercase();
    let components = HEX_REGEX
        .captures(&hex)
        .ok_or(ColorError::InvalidHex(hex.to_owned()))?;
    let r = from_hex!(components.name("r"))?;
    let g = from_hex!(components.name("g"))?;
    let b = from_hex!(components.name("b"))?;
    Ok(Color::rgba(r, g, b, 255))
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Copy, Clone)]
pub struct Color {
    rgba: [u8; 4],
}

#[wasm_bindgen]
impl Color {
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(constructor)]
    pub fn hex(hex: &str) -> Result<Color, JsValue> {
        hex_to_color(hex).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            rgba: [r, g, b, 255],
        }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { rgba: [r, g, b, a] }
    }

    pub fn black() -> Self {
        Self::rgba(0, 0, 0, 255)
    }

    pub fn white() -> Self {
        Self::rgba(255, 255, 255, 255)
    }

    pub fn gray() -> Self {
        Self::rgba(200, 200, 200, 255)
    }
}

impl Into<image::Rgba<u8>> for Color {
    fn into(self) -> image::Rgba<u8> {
        image::Rgba(self.rgba)
    }
}

impl std::str::FromStr for Color {
    type Err = ColorError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex_to_color(s)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Color {
    pub fn hex(hex: &str) -> Result<Color, ColorError> {
        hex_to_color(hex)
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl<'a, P, Container> From<&'a image::ImageBuffer<P, Container>> for Size
where
    P: image::Pixel,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
{
    fn from(image: &'a image::ImageBuffer<P, Container>) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
        }
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl From<OutputSize> for Size {
    fn from(size: OutputSize) -> Self {
        Self {
            width: size.width.unwrap_or(0),
            height: size.height.unwrap_or(0),
        }
    }
}

impl<'a> From<&'a image::DynamicImage> for Size {
    fn from(image: &'a image::DynamicImage) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
        }
    }
}

// impl From<image::DynamicImage> for Size {
//     fn from(image: &image::DynamicImage) -> Self {
//         Self {
//             width: image.width(),
//             height: image.height(),
//         }
//     }
// }

#[derive(Debug, Copy, Clone)]
pub enum CropMode {
    // Custom { x: f64, y: f64 },
    Custom { x: i64, y: i64 },
    Center,
    Bottom,
    Top,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub enum ResizeMode {
    Fill,
    Cover,
    Contain,
}

impl Size {
    pub fn scale_factor<S: Into<Size>>(&self, container: S, mode: ResizeMode) -> (f64, f64) {
        let container = container.into();
        let wratio = container.width as f64 / self.width as f64;
        let hratio = container.height as f64 / self.height as f64;
        match mode {
            ResizeMode::Fill => (wratio, hratio),
            ResizeMode::Cover => (f64::max(wratio, hratio), f64::max(wratio, hratio)),
            ResizeMode::Contain => (f64::min(wratio, hratio), f64::min(wratio, hratio)),
        }
    }

    pub fn min_dim(&self) -> u32 {
        min(self.width, self.height)
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    pub fn center(self, size: Self) -> Rect {
        let container: Point = self.into();
        let size: Point = size.into();
        let top_left = (container - size) / 2.0f64;
        let bottom_right = top_left + size;
        Rect {
            top: top_left.y,
            left: top_left.x,
            bottom: bottom_right.y,
            right: bottom_right.x,
        }
    }

    // pub fn diagonal(&self) -> f64 {
    //     Point::from(*self).magnitude();
    //     let width = (self.width as f64).powi(2);
    //     let height = (self.height as f64).powi(2);
    //     (width + height).sqrt()
    // }

    pub fn clamp<S1, S2>(self, min: S1, max: S2) -> Self
    where
        S1: Into<Size>,
        S2: Into<Size>,
    {
        let min: Size = min.into();
        let max: Size = max.into();
        Self {
            width: utils::clamp(self.width, min.width, max.width),
            height: utils::clamp(self.height, min.height, max.height),
        }
    }

    pub fn scale_by<F, R>(self, scalar: F) -> Self
    where
        R: RoundingMode,
        F: NumCast,
    {
        let scalar: f64 = NumCast::from(scalar).unwrap();
        let width = max(R::round(self.width as f64 * scalar) as u64, 1);
        let height = max(R::round(self.height as f64 * scalar) as u64, 1);
        if width > u32::MAX as u64 {
            let ratio = u32::MAX as f64 / self.width as f64;
            let height = max((self.height as f64 * ratio).round() as u32, 1);
            Self {
                width: u32::MAX,
                height,
            }
        } else if height > u32::MAX as u64 {
            let ratio = u32::MAX as f64 / self.height as f64;
            let width = max((self.width as f64 * ratio).round() as u32, 1);
            Self {
                width,
                height: u32::MAX,
            }
        } else {
            Self {
                width: width as u32,
                height: height as u32,
            }
        }
    }

    // pub fn scale_by<B: Bounded, F: NumCast>(self, scalar: F) -> Self {
    //     let point: Point = self.into();
    //     point.scale_by(scalar).into();
    //     match mode {
    //         ResizeMode::Fill => container,
    //         _ => {
    //             let scale = self.scale_factor(container, mode);
    //             self.scale_by(scale.0)
    //         }
    //     }
    // }

    pub fn scale_to_bounds(self, bounds: OutputSize, mode: ResizeMode) -> Self {
        // let ratio = self.aspect_ratio();
        match bounds {
            // unbounded
            OutputSize {
                width: None,
                height: None,
            } => self,
            // single dimension is bounded
            OutputSize {
                width: None,
                height: Some(height),
            } => {
                self.scale_to(
                    Size {
                        width: self.width,
                        height,
                    },
                    ResizeMode::Contain,
                    // mode,
                )
                // let ratio = height as f64 / output_size.height as f64;
                // let width = output_size.width as f64 * ratio;
                // Size {
                //     width: width as u32,
                //     height,
                // }
            }
            OutputSize {
                width: Some(width),
                height: None,
            } => {
                self.scale_to(
                    Size {
                        width,
                        height: self.height,
                    },
                    ResizeMode::Contain,
                )
                // let ratio = width as f64 / output_size.width as f64;
                // let height = output_size.height as f64 * ratio;
                // Size {
                //     width,
                //     height: height as u32,
                // }
            }
            // all dimensions bounded
            OutputSize {
                width: Some(width),
                height: Some(height),
            } => self.scale_to(Size { width, height }, mode),
        }
        // let container = container.into();
        // match mode {
        //     ResizeMode::Fill => container,
        //     _ => {
        //         let scale = self.scale_factor(container, mode);
        //         self.scale_by::<_, Ceil>(scale.0)
        //     }
        // }
    }

    pub fn scale_to<S: Into<Size>>(self, container: S, mode: ResizeMode) -> Self {
        let container = container.into();
        match mode {
            ResizeMode::Fill => container,
            _ => {
                let scale = self.scale_factor(container, mode);
                self.scale_by::<_, Ceil>(scale.0)
            }
        }
    }

    pub fn crop_to_fit(&self, container: Size, mode: CropMode) -> Sides {
        // avoid underflow if container is larger than self
        let container = container.clamp(Point::origin(), *self);

        crate::debug!(self);
        crate::debug!(&container);
        assert!(self.width >= container.width);
        assert!(self.height >= container.height);

        let center_top_left = self.center(container).top_left();
        // let mut top_left: Point = Point::from(container) - (*self).into();
        // top_left = top_left / 0.5f64;
        // top_left = top_left.scale_by::<_, Round>(0.5f64);

        let top_left: Point = match mode {
            CropMode::Custom { x, y } => center_top_left + Point { x, y },
            CropMode::Right => Point {
                x: self.width as i64 - container.width as i64,
                ..center_top_left
            },
            CropMode::Left => Point {
                x: 0,
                ..center_top_left
            },
            CropMode::Bottom => Point {
                y: self.height as i64 - container.height as i64,
                ..center_top_left
            },
            CropMode::Top => Point {
                y: 0,
                ..center_top_left
            },
            CropMode::Center => center_top_left,
        };
        // do we need a clip to with keep ratio / shift?
        // not really because we compute bottom right anyways without changing the container size

        // let parent_rect: Rect = (*self).into();
        // let crop_rect = Rect::new(top_left, container);
        // let crop_rect = crop_rect.clip_to(&parent_rect);
        // let top_left: Size = crop_rect.top_left().into();

        // this could go wrong but we are careful that top_left +
        let top_left: Size = top_left.into();
        let top_left = top_left.clamp(Point::origin(), *self - container);
        // let bottom_right = *self - (top_left + container);
        // let bottom_right: Size = *self - (top_left + container).into();

        // let bottom_right: Point = (*self - (top_left + container)).into();
        // let bottom_right: Size = *self - (top_left + container);
        // let bottom_right: Size = bottom_right.into();

        // let top_left: Size = top_left.into();
        let bottom_right = top_left + container;
        let bottom_right = bottom_right.clamp(Point::origin(), *self);
        let bottom_right = *self - bottom_right;

        Sides {
            // top: top_left.y as u32,
            // left: top_left.x as u32,
            // bottom: bottom_right.y as u32,
            // right: bottom_right.x as u32,
            top: top_left.height,
            left: top_left.width,
            bottom: bottom_right.height,
            right: bottom_right.width,
        }
    }
}

impl std::ops::Sub<u32> for Size {
    type Output = Self;

    fn sub(self, scalar: u32) -> Self::Output {
        Self {
            width: self.width - scalar,
            height: self.height - scalar,
        }
    }
}

impl std::ops::Add<u32> for Size {
    type Output = Self;

    fn add(self, scalar: u32) -> Self::Output {
        Self {
            width: self.width + scalar,
            height: self.height + scalar,
        }
    }
}

impl<F> std::ops::Mul<F> for Size
where
    F: NumCast,
{
    type Output = Self;

    fn mul(self, scalar: F) -> Self::Output {
        self.scale_by::<_, Round>(scalar)
    }
}

impl<F> std::ops::Div<F> for Size
where
    F: NumCast,
{
    type Output = Self;

    fn div(self, scalar: F) -> Self::Output {
        let scalar: f64 = NumCast::from(scalar).unwrap();
        self.scale_by::<_, Round>(1.0 / scalar)
    }
}

impl std::ops::Sub<Sides> for Size {
    type Output = Self;

    fn sub(self, sides: Sides) -> Self::Output {
        let width = self.width as i64 - sides.width() as i64;
        let height = self.height as i64 - sides.height() as i64;
        // let top_left =
        // let width = self.width as i64 - top_left
        // width = width - sides.left as i64 - sides.right as i64;
        // let width = self.width as i64 - sides.left as i64 - sides.right as i64;
        Size {
            width: utils::clamp(width, 0, u32::MAX as i64) as u32,
            height: utils::clamp(height, 0, u32::MAX as i64) as u32,
        }
    }
}

impl std::ops::Add<Sides> for Size {
    type Output = Self;

    fn add(self, sides: Sides) -> Self::Output {
        let width = self.width as i64 + sides.width() as i64;
        let height = self.height as i64 + sides.height() as i64;
        Size {
            width: utils::clamp(width, 0, u32::MAX as i64) as u32,
            height: utils::clamp(height, 0, u32::MAX as i64) as u32,
            // height: self.height + sides.top + sides.bottom,
            // width: self.width + sides.left + sides.right,
            // height: self.height + sides.top + sides.bottom,
        }
    }
}

impl std::ops::Add<Point> for Size {
    type Output = Self;

    fn add(self, p: Point) -> Self::Output {
        let width = utils::clamp(self.width as i64 + p.x, 0, u32::MAX as i64);
        let height = utils::clamp(self.height as i64 + p.y, 0, u32::MAX as i64);
        Size {
            width: width as u32,
            height: height as u32,
        }
    }
}

impl std::ops::Add for Size {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl std::ops::Sub for Size {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Size {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl From<Sides> for Size {
    fn from(sides: Sides) -> Self {
        Self {
            width: sides.left + sides.right,
            height: sides.top + sides.bottom,
        }
    }
}

impl From<Point> for Size {
    fn from(point: Point) -> Self {
        Self {
            width: utils::clamp(point.x, 0, u32::MAX as i64) as u32,
            height: utils::clamp(point.y, 0, u32::MAX as i64) as u32,
        }
    }
}

#[wasm_bindgen]
impl Size {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Size::default()
    }

    pub fn max(&self) -> u32 {
        max(self.width, self.height)
    }

    pub fn min(&self) -> u32 {
        min(self.width, self.height)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Vector<I> {
    pub x: I,
    pub y: I,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
// pub struct Point<I> {
pub struct Point {
    pub x: i64,
    pub y: i64,
}

// impl<I> std::ops::Add for Point<I> {
impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

// impl<I> std::ops::Sub for Point<I> {
impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

// impl<I> std::ops::Add<Size> for Point<I> {
// impl std::ops::Add<Size> for Point<i64> {
impl std::ops::Add<Size> for Point {
    type Output = Self;

    fn add(self, size: Size) -> Self::Output {
        Point {
            // x: NumCast::from(self.x + size.width as i64,
            x: self.x + size.width as i64,
            y: self.y + size.height as i64,
        }
    }
}

// impl<I, F> std::ops::Mul<F> for Point<I>
impl<F> std::ops::Mul<F> for Point
where
    F: NumCast,
{
    type Output = Self;

    fn mul(self, scalar: F) -> Self::Output {
        self.scale_by::<_, Round>(scalar)
    }
}

// impl<I, F> std::ops::Div<F> for Point<I>
impl<F> std::ops::Div<F> for Point
where
    F: NumCast,
{
    type Output = Self;

    fn div(self, scalar: F) -> Self::Output {
        let scalar: f64 = NumCast::from(scalar).unwrap();
        self.scale_by::<_, Round>(1.0 / scalar)
    }
}

// impl<I, S> From<cgmath::Point2<S>> for Point<I>
// where
//     S: cgmath::BaseNum,
// {
//     fn from(point: cgmath::Point2<S>) -> Self {
//         Self {
//             x: NumCast::from(point.x).unwrap(),
//             y: NumCast::from(point.y).unwrap(),
//         }
//     }
// }

// impl<S> Into<cgmath::Point2<S>> for Point<I>
// where
//     S: cgmath::BaseNum,
// {
//     fn into(self) -> cgmath::Point2<S> {
//         cgmath::Point2 {
//             x: NumCast::from(self.x).unwrap(),
//             y: NumCast::from(self.y).unwrap(),
//         }
//     }
// }

// impl From<Size> for Point<i64> {
impl From<Size> for Point {
    fn from(size: Size) -> Self {
        Self {
            x: size.width as i64,
            y: size.height as i64,
        }
    }
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Point::origin()
    }

    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Point {
    pub fn clamp<P1, P2>(self, min: P1, max: P2) -> Self
    where
        P1: Into<Point>,
        P2: Into<Point>,
    {
        let min: Point = min.into();
        let max: Point = max.into();
        Self {
            x: utils::clamp(self.x, min.x, max.x),
            y: utils::clamp(self.y, min.y, max.y),
        }
    }

    pub fn scale_by<F, R>(self, scalar: F) -> Self
    where
        R: RoundingMode,
        F: NumCast,
    {
        let scalar: f64 = NumCast::from(scalar).unwrap();
        let x = R::round(self.x as f64 * scalar) as u64;
        let y = R::round(self.y as f64 * scalar) as u64;
        Self {
            x: x as i64,
            y: y as i64,
        }
        // if x > u32::MAX as u64 {
        //     let ratio = u32::MAX as f64 / self.x as f64;
        //     let y = max((self.y as f64 * ratio).ceil() as u32, 1);
        //     Self { x: u32::MAX, y }
        // } else if y > u32::MAX as u64 {
        //     let ratio = u32::MAX as f64 / self.y as f64;
        //     let x = max((self.x as f64 * ratio).ceil() as u32, 1);
        //     Self { x, y: u32::MAX }
        // } else {
        //     Self {
        //         x: x as u32,
        //         y: y as u32,
        //     }
        // }
    }

    pub fn unit_vector(self) -> Vector<f64> {
        let mag = self.magnitude();
        Vector {
            x: self.x as f64 / mag,
            y: self.y as f64 / mag,
        }
    }

    pub fn magnitude(&self) -> f64 {
        // c**2 = a**2 + b**2
        let x = (self.x as f64).powi(2);
        let y = (self.y as f64).powi(2);
        (x + y).sqrt()
        // (self.x.powi(2) + self.y.powi(2)).sqrt()
        // let point: cgmath::Point2<f64> = self.into();
        // let rot: cgmath::Basis2<f64> = rotation.into();
        // rot.rotate_point(point).into()
    }

    // pub fn rotate(self, rotation: Rotation) -> Self {
    //     let point: cgmath::Point2<f64> = self.into();
    //     let rot: cgmath::Basis2<f64> = rotation.into();
    //     rot.rotate_point(point).into()
    // }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct SidesPercent {
    pub top: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
}

fn percent_to_abs(percent: f32, dimension: u32) -> u32
// fn percent_to_abs<I>(percent: f32, dimension: I) -> I
// where
//     I: NumCast,
{
    let percent = percent.max(0.0);
    // let dimension: f32 = NumCast::from(dimension).unwrap();

    if percent <= 1.0 {
        let absolute = percent * dimension as f32;
        utils::clamp(absolute, 0.0, dimension as f32).ceil() as u32
    } else {
        utils::clamp(percent, 0.0, dimension as f32).ceil() as u32
    }
    // NumCast::from(abs).unwrap()
}

impl std::ops::Mul<u32> for SidesPercent
// impl<I> std::ops::Mul<I> for SidesPercent
// where
//     I: NumCast,
{
    type Output = Sides;

    fn mul(self, scalar: u32) -> Self::Output {
        Self::Output {
            top: percent_to_abs(self.top, scalar),
            left: percent_to_abs(self.left, scalar),
            bottom: percent_to_abs(self.bottom, scalar),
            right: percent_to_abs(self.right, scalar),
        }
    }
}

impl std::ops::Mul<Size> for SidesPercent {
    type Output = Sides;

    fn mul(self, size: Size) -> Self::Output {
        Self::Output {
            top: percent_to_abs(self.top, size.height),
            left: percent_to_abs(self.left, size.width),
            bottom: percent_to_abs(self.bottom, size.height),
            right: percent_to_abs(self.right, size.width),
        }
    }
}

#[wasm_bindgen]
impl SidesPercent {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn uniform(side: f32) -> Self {
        Self {
            top: side,
            left: side,
            right: side,
            bottom: side,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Sides {
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
}

impl Sides {
    pub fn uniform(side: u32) -> Self {
        Self {
            top: side,
            left: side,
            right: side,
            bottom: side,
        }
    }
    // pub fn padding(value: u32) -> Self {
    //     Self {
    //         top: -value,
    //         left: -value,
    //         right: value,
    //         bottom: value,
    //     }
    // }

    pub fn height(&self) -> u32 {
        self.top + self.bottom
    }

    pub fn width(&self) -> u32 {
        self.left + self.right
    }

    pub fn top_left(&self) -> Point {
        Point {
            x: self.left as i64,
            y: self.top as i64,
        }
    }

    pub fn bottom_right(&self) -> Point {
        Point {
            x: self.right as i64,
            y: self.bottom as i64,
        }
    }
}

impl std::ops::Add for Sides {
    type Output = Self;

    fn add(self, side: Self) -> Self::Output {
        Self {
            top: self.top + side.top,
            right: self.right + side.right,
            bottom: self.bottom + side.bottom,
            left: self.left + side.left,
        }
    }
}

// impl std::ops::Add<u32> for Sides {
//     type Output = Self;

//     fn add(self, scalar: u32) -> Self::Output {
//         Self {
//             top: self.top + scalar,
//             right: self.right + scalar,
//             bottom: self.bottom + scalar,
//             left: self.left + scalar,
//         }
//     }
// }

impl<F> std::ops::Mul<F> for Sides
where
    F: NumCast,
{
    type Output = Self;

    fn mul(self, scalar: F) -> Self::Output {
        let scalar: f32 = NumCast::from(scalar).unwrap();
        // let scalar = max(0.0, scalar);
        Self {
            top: (self.top as f32 * scalar).ceil() as u32,
            right: (self.right as f32 * scalar).ceil() as u32,
            bottom: (self.bottom as f32 * scalar).ceil() as u32,
            left: (self.left as f32 * scalar).ceil() as u32,
        }
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl<F> Into<cgmath::Basis2<F>> for Rotation
where
    F: cgmath::BaseFloat,
{
    fn into(self) -> cgmath::Basis2<F> {
        let angle: F = match self {
            Rotation::Rotate270 => NumCast::from(270.0).unwrap(),
            Rotation::Rotate180 => NumCast::from(180.0).unwrap(),
            Rotation::Rotate90 => NumCast::from(90.0).unwrap(),
            Rotation::Rotate0 => NumCast::from(0.0).unwrap(),
        };
        cgmath::Rotation2::from_angle(cgmath::Deg(angle))
    }
}

impl std::str::FromStr for Rotation {
    type Err = ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "270" => Ok(Rotation::Rotate270),
            "180" => Ok(Rotation::Rotate180),
            "90" => Ok(Rotation::Rotate90),
            "0" => Ok(Rotation::Rotate0),
            _ => Err(ParseEnumError::Unknown(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Border, BorderOptions, Color, Point, Rect, Rotation};
    use crate::error::{BorderError, Error};
    use crate::{imageops, img};
    use anyhow::Result;
    use std::path::{Path, PathBuf};

    macro_rules! color_hex_tests {
        ($($name:ident: $values:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (hex, rgba) = $values;
                    assert_eq!(Color::hex(hex).ok(), rgba);
                }
            )*
        }
    }

    color_hex_tests! {
        test_parse_valid_hex_color_1: (
            "#4287f5", Some(Color::rgba(66, 135, 245, 255))),
        test_parse_valid_hex_color_2: (
            "4287f5", Some(Color::rgba(66, 135, 245, 255))),
        test_parse_valid_hex_color_3: (
            "  # 4287f5  ", Some(Color::rgba(66, 135, 245, 255))),
        test_parse_valid_hex_color_4: (
            "#e942f5", Some(Color::rgba(233, 66, 245, 255))),
        test_parse_valid_hex_color_5: (
            "  e942f5", Some(Color::rgba(233, 66, 245, 255))),
        test_parse_invalid_hex_color_1: ("  # 487f5  ", None),
        test_parse_invalid_hex_color_2: ("487f5", None),
        test_parse_invalid_hex_color_3: ("#e942g5", None),
    }

    fn draw_transparent_components<P: AsRef<Path>>(
        mut border: img::Image,
        components: &Vec<Component>,
        output: P,
    ) -> Result<()> {
        let alpha = (255.0f32 * 0.5).ceil() as u8;
        let red = Color::rgba(255, 0, 0, alpha).to_rgba();
        for c in components {
            let top_left = Point {
                y: c.top,
                x: c.left,
            };
            let bottom_right = Point {
                y: c.bottom,
                x: c.right,
            };
            let size = bottom_right - top_left;
            imageops::fill_rect(&mut border, &red, top_left, size);
        }
        border.save(Some(&output), None)?;
        Ok(())
    }

    macro_rules! transparent_areas_tests {
        ($($name:ident: $values:expr,)*) => {
            $(
                #[test]
                fn $name() -> Result<()> {
                    let (border_path, expected_components) = $values;
                    let repo: PathBuf = env!("CARGO_MANIFEST_DIR").into();
                    let border_file = repo.join(&border_path);
                    let options = BorderOptions::default();
                    let img = img::Image::open(&border_file)?;
                    let border = Border::from_image(img.clone(), Some(options));
                    let components = match border {
                        Err(Error::Border(BorderError::BadTransparency(c))) => Ok(c),
                        Err(err) => Err(err),
                        Ok(border) => {
                            Ok(border.transparent_components().to_vec())
                        }
                    }?;

                    // debug components
                    let output = repo.join(
                        format!("testing/{}.png", stringify!($name)));
                    draw_transparent_components(img, &components, &output)?;
                    println!("components: {:?}", components);

                    assert_eq!(components.len(), expected_components);
                    Ok(())
                }
            )*
        }
    }

    transparent_areas_tests! {
        test_transparent_areas_3_vertical: (
            "samples/borders/border_3_areas_vertical.png", 3),
        test_transparent_areas_3_horizontal: (
            "samples/borders/border_3_areas_horizontal.png", 3),
        test_transparent_areas_1_vertical: (
            "samples/borders/border_1_areas_vertical.png", 1),
        test_transparent_areas_1_horizontal: (
            "samples/borders/border_1_areas_horizontal.png", 1),
    }

    #[test]
    fn test_transparent_areas_3_rotate() -> Result<()> {
        let repo: PathBuf = env!("CARGO_MANIFEST_DIR").into();
        let border_file = repo.join("samples/borders/border_3_areas_vertical.png");
        let options = BorderOptions::default();
        let options = BorderOptions {
            transparent_component_threshold: 8,
            alpha_threshold: 0.95,
        };
        let img = img::Image::open(&border_file)?;
        let border = Border::from_image(img.clone(), Some(options))?;

        for rotation in vec![
            Rotation::Rotate0,
            Rotation::Rotate90,
            Rotation::Rotate180,
            Rotation::Rotate270,
        ] {
            let mut rotated = border.clone();
            rotated.rotate(rotation);
            let output = repo.join(format!(
                "testing/border_3_areas_vertical_{:?}.png",
                rotation
            ));
            let components = rotated.transparent_components().to_vec();
            println!("sorted components: {:?}", &components);
            draw_transparent_components(rotated.inner, &components, &output)?;
            assert_eq!(components.len(), 3);
        }
        Ok(())
    }

    #[test]
    fn test_output_size() {
        use super::utils::opt_min;
        use super::OutputSize;
        assert_eq!(opt_min(Some(10), Some(5)), Some(5));
        assert_eq!(opt_min(Some(10), Some(15)), Some(10));
        assert_eq!(opt_min(None::<u32>, Some(15)), None);
        assert_eq!(opt_min(None::<u32>, None), None);
        assert_eq!(opt_min(Some(10), None), Some(10));
        assert_eq!(
            OutputSize {
                width: Some(10),
                height: None
            }
            .min(OutputSize {
                width: Some(12),
                height: Some(10)
            }),
            OutputSize {
                width: Some(10),
                height: None
            }
        );
        assert_eq!(
            OutputSize {
                width: Some(10),
                height: None
            }
            .min(OutputSize {
                width: Some(5),
                height: Some(10)
            }),
            OutputSize {
                width: Some(5),
                height: None
            }
        );
    }

    #[ignore]
    #[test]
    fn try_rotations() -> Result<()> {
        use anyhow;
        use cgmath::*;
        let unit_x: Vector2<f32> = Vector2::unit_y();
        println!("{:?}", &unit_x);
        let point: Point2<f32> = Point2 { x: 0f32, y: 1f32 };
        println!("{:?}", &point);
        let rot: Basis2<f32> = Rotation2::from_angle(Deg(90.0f32));
        println!("{:?}", &rot);

        let test = rot.rotate_point(point);
        println!("{:?}", &test);
        println!("{:?}", &rot.rotate_vector(unit_x));

        return Err(anyhow::anyhow!("does not work yet"));
        Ok(())
    }
}
