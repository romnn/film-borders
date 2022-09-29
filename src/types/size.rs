use super::*;
use crate::error::*;
use crate::imageops::*;
use crate::numeric::{Ceil, Round, RoundingMode};
use crate::{img, utils};
use num::traits::NumCast;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::path::Path;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Copy, Clone)]
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

impl From<BoundedSize> for Size {
    fn from(size: BoundedSize) -> Self {
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

    #[inline]
    pub fn min_dim(&self) -> u32 {
        min(self.width, self.height)
    }

    #[inline]
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    #[inline]
    pub fn is_portrait(&self) -> bool {
        self.orientation() == Orientation::Portrait
    }

    #[inline]
    pub fn orientation(&self) -> Orientation {
        if self.width <= self.height {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        }
    }

    #[inline]
    pub fn rotate(self, angle: Rotation) -> Self {
        match angle {
            Rotation::Rotate0 | Rotation::Rotate180 => self,
            Rotation::Rotate90 | Rotation::Rotate270 => Self {
                width: self.height,
                height: self.width,
            },
        }
    }

    #[inline]
    pub fn rotate_to_orientation(self, orientation: Orientation) -> Size {
        self.rotate(if self.orientation() != orientation {
            Rotation::Rotate90
        } else {
            Rotation::Rotate0
        })
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

    pub fn scale_to_bounds(self, bounds: BoundedSize, mode: ResizeMode) -> Self {
        match bounds {
            // unbounded
            BoundedSize {
                width: None,
                height: None,
            } => self,
            // single dimension is bounded
            BoundedSize {
                width: None,
                height: Some(height),
            } => self.scale_to(
                Size {
                    width: self.width,
                    height,
                },
                mode,
            ),
            BoundedSize {
                width: Some(width),
                height: None,
            } => self.scale_to(
                Size {
                    width,
                    height: self.height,
                },
                mode,
            ),
            // all dimensions bounded
            BoundedSize {
                width: Some(width),
                height: Some(height),
            } => self.scale_to(Size { width, height }, mode),
        }
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

        assert!(self.width >= container.width);
        assert!(self.height >= container.height);

        let center_top_left = self.center(container).top_left();

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
        // this could go wrong but we are careful
        let top_left: Size = top_left.into();
        let top_left = top_left.clamp(Point::origin(), *self - container);

        let bottom_right = top_left + container;
        let bottom_right = bottom_right.clamp(Point::origin(), *self);
        let bottom_right = *self - bottom_right;

        Sides {
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
        Self::default()
    }

    pub fn wh(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn max(&self) -> u32 {
        max(self.width, self.height)
    }

    pub fn min(&self) -> u32 {
        min(self.width, self.height)
    }
}
