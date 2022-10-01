use super::sides::abs::Sides;
use crate::numeric::ops::{self, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use crate::numeric::{self, error, Cast, Ceil, Round};
use crate::Error;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl numeric::Numeric for Size {}

#[wasm_bindgen]
impl Size {
    #[wasm_bindgen(constructor)]
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Size {
    #[inline]
    pub fn scale_factor<S: Into<Size>>(&self, container: S, mode: super::ResizeMode) -> (f64, f64) {
        use super::ResizeMode;
        let container = container.into();
        let width_ratio = f64::from(container.width) / f64::from(self.width);
        let height_ratio = f64::from(container.height) / f64::from(self.height);
        match mode {
            ResizeMode::Fill => (width_ratio, height_ratio),
            ResizeMode::Cover => (
                f64::max(width_ratio, height_ratio),
                f64::max(width_ratio, height_ratio),
            ),
            ResizeMode::Contain => (
                f64::min(width_ratio, height_ratio),
                f64::min(width_ratio, height_ratio),
            ),
        }
    }

    #[inline]
    #[must_use]
    pub fn max_dim(&self) -> u32 {
        self.width.max(self.height)
    }

    #[inline]
    #[must_use]
    pub fn min_dim(&self) -> u32 {
        self.width.min(self.height)
    }

    #[inline]
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        f64::from(self.width) / f64::from(self.height)
    }

    #[inline]
    #[must_use]
    pub fn is_portrait(&self) -> bool {
        self.orientation() == super::Orientation::Portrait
    }

    #[inline]
    #[must_use]
    pub fn orientation(&self) -> super::Orientation {
        use super::Orientation;
        if self.width <= self.height {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        }
    }

    #[inline]
    #[must_use]
    pub fn rotate(self, angle: super::Rotation) -> Self {
        use super::Rotation;
        match angle {
            Rotation::Rotate0 | Rotation::Rotate180 => self,
            Rotation::Rotate90 | Rotation::Rotate270 => Self {
                width: self.height,
                height: self.width,
            },
        }
    }

    #[inline]
    #[must_use]
    pub fn rotate_to_orientation(self, orientation: super::Orientation) -> Size {
        use super::Rotation;
        self.rotate(if self.orientation() == orientation {
            Rotation::Rotate0
        } else {
            Rotation::Rotate90
        })
    }

    #[inline]
    #[must_use]
    pub fn center(self, size: Self) -> super::Rect {
        let container: super::Point = self.into();
        let size: super::Point = size.into();
        let top_left = container
            .checked_sub(size)
            .unwrap()
            .checked_div(2.0)
            .unwrap();
        let bottom_right = top_left.checked_add(size).unwrap();
        super::Rect {
            top: top_left.y,
            left: top_left.x,
            bottom: bottom_right.y,
            right: bottom_right.x,
        }
    }

    #[inline]
    #[must_use]
    pub fn clamp<S1, S2>(self, min: S1, max: S2) -> Self
    where
        S1: Into<Size>,
        S2: Into<Size>,
    {
        let min: Size = min.into();
        let max: Size = max.into();
        Self {
            width: self.width.clamp(min.width, max.width),
            height: self.height.clamp(min.height, max.height),
        }
    }

    #[inline]
    pub fn scale_by<F, R>(self, scalar: F) -> Result<Self, numeric::Error>
    where
        R: numeric::RoundingMode,
        F: numeric::Cast + numeric::Numeric,
    {
        let scalar = scalar.cast::<f64>()?;
        let width = self.width.cast::<f64>()?;
        let height = self.height.cast::<f64>()?;
        let width = CheckedMul::checked_mul(width, scalar)?;
        let height = CheckedMul::checked_mul(height, scalar)?;
        // todo: should we allow the size to go zero here?
        let width = R::round(width).cast::<u32>()?;
        let height = R::round(height).cast::<u32>()?;
        Ok(Self { width, height })
    }

    #[inline]
    #[must_use]
    pub fn scale_to_bounds(self, bounds: super::BoundedSize, mode: super::ResizeMode) -> Self {
        match bounds {
            // unbounded
            super::BoundedSize {
                width: None,
                height: None,
            } => self,
            // single dimension is bounded
            super::BoundedSize {
                width: None,
                height: Some(height),
            } => self.scale_to(
                Size {
                    width: self.width,
                    height,
                },
                mode,
            ),
            super::BoundedSize {
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
            super::BoundedSize {
                width: Some(width),
                height: Some(height),
            } => self.scale_to(Size { width, height }, mode),
        }
    }

    #[inline]
    #[must_use]
    pub fn scale_to<S: Into<Size>>(self, container: S, mode: super::ResizeMode) -> Self {
        let container = container.into();
        if mode == super::ResizeMode::Fill {
            container
        } else {
            let scale = self.scale_factor(container, mode);
            self.scale_by::<_, Ceil>(scale.0).unwrap()
        }
    }

    #[inline]
    #[must_use]
    pub fn crop_to_fit(&self, container: Size, mode: super::CropMode) -> Sides {
        use super::{CropMode, Point};
        // avoid underflow if container is larger than self
        let container = container.clamp((0, 0), *self);

        assert!(self.width >= container.width);
        assert!(self.height >= container.height);

        let center_top_left = self.center(container).top_left();

        let top_left: Point = match mode {
            CropMode::Custom { x, y } => center_top_left.checked_add(Point { x, y }).unwrap(),
            CropMode::Right => Point {
                x: i64::from(self.width)
                    .checked_sub(i64::from(container.width))
                    .unwrap(),
                ..center_top_left
            },
            CropMode::Left => Point {
                x: 0,
                ..center_top_left
            },
            CropMode::Bottom => Point {
                y: i64::from(self.height)
                    .checked_sub(i64::from(container.height))
                    .unwrap(),
                ..center_top_left
            },
            CropMode::Top => Point {
                y: 0,
                ..center_top_left
            },
            CropMode::Center => center_top_left,
        };
        // this could go wrong but we are careful
        let top_left: Size = top_left.try_into().unwrap();
        let top_left = top_left.clamp((0, 0), self.checked_sub(container).unwrap());

        let bottom_right = top_left.checked_add(container).unwrap();
        let bottom_right = bottom_right.clamp((0, 0), *self);
        let bottom_right = self.checked_sub(bottom_right).unwrap();

        Sides {
            top: top_left.height,
            left: top_left.width,
            bottom: bottom_right.height,
            right: bottom_right.width,
        }
    }
}

impl std::fmt::Display for Size {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl<'a, P, Container> From<&'a image::ImageBuffer<P, Container>> for Size
where
    P: image::Pixel,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
{
    #[inline]
    fn from(image: &'a image::ImageBuffer<P, Container>) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
        }
    }
}

impl<'a> From<&'a image::DynamicImage> for Size {
    #[inline]
    fn from(image: &'a image::DynamicImage) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
        }
    }
}

impl From<(u32, u32)> for Size {
    #[inline]
    fn from(size: (u32, u32)) -> Self {
        Self {
            width: size.0,
            height: size.1,
        }
    }
}

impl TryFrom<super::Point> for Size {
    // type Error = numeric::CastError<super::Point, Size>;
    type Error = Error;

    #[inline]
    fn try_from(point: super::Point) -> Result<Self, Self::Error> {
        let size = (|| {
            let width = point.x.cast::<u32>()?;
            let height = point.y.cast::<u32>()?;
            Ok::<Size, numeric::CastError<i64, u32>>(Self { width, height })
        })();
        size.map_err(|err| numeric::CastError {
            src: point,
            target: std::marker::PhantomData,
            cause: Some(Box::new(err)),
        })
        .map_err(
            |err: numeric::CastError<super::Point, Size>| Error::Arithmetic {
                msg: format!("failed to convert {} to size", point),
                source: err.into(),
            },
        )
    }
}

impl<F> CheckedMul<F> for Size
where
    F: numeric::Cast + numeric::Numeric,
{
    type Output = Self;
    type Error = ops::MulError<Self, F>;

    #[inline]
    fn checked_mul(self, scalar: F) -> Result<Self::Output, Self::Error> {
        match self.scale_by::<_, Round>(scalar) {
            Ok(size) => Ok(size),
            Err(numeric::Error(err)) => Err(ops::MulError(error::Arithmetic {
                lhs: self,
                rhs: scalar,
                kind: None,
                cause: Some(err),
            })),
        }
    }
}

impl<F> CheckedDiv<F> for Size
where
    F: numeric::Cast + numeric::Numeric + num::traits::Inv<Output = F>,
{
    type Output = Self;
    type Error = ops::DivError<Self, F>;

    #[inline]
    fn checked_div(self, scalar: F) -> Result<Self::Output, Self::Error> {
        let inverse = scalar.inv();
        match self.scale_by::<_, Round>(inverse) {
            Ok(size) => Ok(size),
            Err(numeric::Error(err)) => Err(ops::DivError(error::Arithmetic {
                lhs: self,
                rhs: inverse,
                kind: None,
                cause: Some(err),
            })),
        }
    }
}

impl CheckedSub<Sides> for Size {
    type Output = Self;
    type Error = ops::SubError<Self, Sides>;

    #[inline]
    fn checked_sub(self, rhs: Sides) -> Result<Self::Output, Self::Error> {
        match (|| {
            let width = CheckedSub::checked_sub(self.width, rhs.width())?;
            let height = CheckedSub::checked_sub(self.height, rhs.height())?;
            Ok::<Self, ops::SubError<u32, u32>>(Self { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(ops::SubError(error::Arithmetic {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}

impl CheckedAdd<Sides> for Size {
    type Output = Self;
    type Error = ops::AddError<Self, Sides>;

    #[inline]
    fn checked_add(self, rhs: Sides) -> Result<Self::Output, Self::Error> {
        match (|| {
            let width = CheckedAdd::checked_add(self.width, rhs.width())?;
            let height = CheckedAdd::checked_add(self.height, rhs.height())?;
            Ok::<Self, ops::AddError<u32, u32>>(Self { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(ops::AddError(error::Arithmetic {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}

impl CheckedAdd for Size {
    type Output = Self;
    type Error = ops::AddError<Self, Self>;

    #[inline]
    fn checked_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        match (|| {
            let width = CheckedAdd::checked_add(self.width, rhs.width)?;
            let height = CheckedAdd::checked_add(self.height, rhs.height)?;
            Ok::<Self, ops::AddError<u32, u32>>(Self { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(ops::AddError(error::Arithmetic {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}

impl CheckedSub for Size {
    type Output = Self;
    type Error = ops::SubError<Self, Self>;

    #[inline]
    fn checked_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        match (|| {
            let width = CheckedSub::checked_sub(self.width, rhs.width)?;
            let height = CheckedSub::checked_sub(self.height, rhs.height)?;
            Ok::<Self, ops::SubError<u32, u32>>(Self { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(ops::SubError(error::Arithmetic {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}
