use super::sides::abs::Sides;
use crate::arithmetic::{
    self,
    ops::{self, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
    Cast, CastError, Ceil, Clamp, ClampMin, Round,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to compute scale factors to scale {size} to {target} with mode {mode:?}")]
pub struct ScaleFactorsError {
    size: Size,
    target: Size,
    mode: super::ResizeMode,
    source: ops::DivError<f64, f64>,
}

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum ScaleToError {
    #[error(transparent)]
    ScaleFactors(#[from] ScaleFactorsError),

    #[error(transparent)]
    ScaleBy(#[from] ScaleByError),
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to compute crop such that {size} fits {container}")]
pub struct CropToFitError {
    size: Size,
    container: Size,
    source: CropToFitErrorSource,
}

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum CropToFitErrorSource {
    #[error(transparent)]
    Center(#[from] CenterError),

    #[error(transparent)]
    Arithmetic(#[from] arithmetic::Error),
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to scale {size} to bounds {bounds:?} with mode {:?}")]
pub struct ScaleToBoundsError {
    size: Size,
    bounds: super::BoundedSize,
    mode: super::ResizeMode,
    source: ScaleToError,
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to scale {size} by {scalar:?}")]
pub struct ScaleByError {
    size: Size,
    scalar: Option<f64>,
    source: arithmetic::Error,
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to compute aspect ratio of {size}")]
pub struct AspectRatioError {
    size: Size,
    source: ops::DivError<f64, f64>,
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to center {child} in {parent}")]
pub struct CenterError {
    child: Size,
    parent: Size,
    source: arithmetic::Error,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl arithmetic::Type for Size {}

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
    pub fn scale_factor(
        &self,
        size: impl Into<Size>,
        mode: super::ResizeMode,
    ) -> Result<(f64, f64), ScaleFactorsError> {
        let target = size.into();
        match (|| {
            let target_width = f64::from(target.width);
            let width = f64::from(self.width);
            let target_height = f64::from(target.height);
            let height = f64::from(self.height);

            let width_ratio = target_width.checked_div(width)?;
            let height_ratio = target_height.checked_div(height)?;

            use super::ResizeMode;
            let factors = match mode {
                ResizeMode::Fill => (width_ratio, height_ratio),
                ResizeMode::Cover => (
                    f64::max(width_ratio, height_ratio),
                    f64::max(width_ratio, height_ratio),
                ),
                ResizeMode::Contain => (
                    f64::min(width_ratio, height_ratio),
                    f64::min(width_ratio, height_ratio),
                ),
            };
            Ok::<(f64, f64), ops::DivError<f64, f64>>(factors)
        })() {
            Ok(factors) => Ok(factors),
            Err(err) => Err(ScaleFactorsError {
                size: *self,
                target,
                mode,
                source: err,
            }),
        }
    }

    #[inline]
    #[must_use]
    pub fn contains(&self, point: &super::Point) -> bool {
        let rect = super::Rect::from(*self);
        rect.contains(&point)
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
    pub fn aspect_ratio(&self) -> Result<f64, AspectRatioError> {
        f64::from(self.width)
            .checked_div(f64::from(self.height))
            .map_err(|err| AspectRatioError {
                size: *self,
                source: err,
            })
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
    pub fn center(self, size: Self) -> Result<super::Rect, CenterError> {
        let parent: super::Point = self.into();
        let child: super::Point = size.into();
        match (|| {
            let top_left = parent.checked_sub(child)?.checked_div(2.0)?;
            let bottom_right = top_left.checked_add(child)?;
            let centered = super::Rect::from_points(top_left, bottom_right);
            Ok(centered)
        })() {
            Ok(rect) => Ok(rect),
            Err(err) => Err(CenterError {
                child: size,
                parent: self,
                source: err,
            }),
        }
    }

    #[inline]
    pub fn scale_by<F, R>(self, scalar: F) -> Result<Self, ScaleByError>
    where
        R: arithmetic::RoundingMode,
        F: arithmetic::Cast + arithmetic::Type,
    {
        match (|| {
            let scalar = scalar.cast::<f64>()?;
            let width = self.width.cast::<f64>()?;
            let height = self.height.cast::<f64>()?;
            let width = CheckedMul::checked_mul(width, scalar)?;
            let height = CheckedMul::checked_mul(height, scalar)?;
            // todo: should we allow the size to go zero here?
            let width = R::round(width).cast::<u32>()?;
            let height = R::round(height).cast::<u32>()?;
            Ok::<Self, arithmetic::Error>(Self { width, height })
        })() {
            Ok(scaled_size) => Ok(scaled_size),
            Err(err) => Err(ScaleByError {
                size: self,
                scalar: scalar.cast::<f64>().ok(),
                source: err,
            }),
        }
    }

    #[inline]
    pub fn scale_to_bounds(
        self,
        bounds: super::BoundedSize,
        mode: super::ResizeMode,
    ) -> Result<Self, ScaleToBoundsError> {
        match (|| {
            match bounds {
                // unbounded
                super::BoundedSize {
                    width: None,
                    height: None,
                } => Ok(self),
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
        })() {
            Ok(scaled_size) => Ok(scaled_size),
            Err(err) => Err(ScaleToBoundsError {
                size: self,
                bounds,
                mode,
                source: err,
            }),
        }
    }

    #[inline]
    pub fn scale_to<S: Into<Size>>(
        self,
        size: S,
        mode: super::ResizeMode,
    ) -> Result<Self, ScaleToError> {
        let target = size.into();
        if mode == super::ResizeMode::Fill {
            Ok(target)
        } else {
            let scale = self.scale_factor(target, mode)?;
            let scaled = self.scale_by::<_, Ceil>(scale.0)?;
            Ok(scaled)
        }
    }

    #[inline]
    pub fn crop_to_fit(&self, size: Size, mode: super::CropMode) -> Result<Sides, CropToFitError> {
        use super::{CropMode, Point};

        let center = self.center(size).map_err(|err| CropToFitError {
            size: *self,
            container: size,
            source: err.into(),
        })?;

        let center_top_left = center.top_left();
        match (|| {
            let top_left: Point = match mode {
                CropMode::Custom { x, y } => center_top_left.checked_add(Point { x, y })?,
                CropMode::Right => Point {
                    x: CheckedSub::checked_sub(i64::from(self.width), i64::from(size.width))?,
                    ..center_top_left
                },
                CropMode::Left => Point {
                    x: 0,
                    ..center_top_left
                },
                CropMode::Bottom => Point {
                    y: CheckedSub::checked_sub(i64::from(self.height), i64::from(size.height))?,
                    ..center_top_left
                },
                CropMode::Top => Point {
                    y: 0,
                    ..center_top_left
                },
                CropMode::Center => center_top_left,
            };

            let max_top_left = Point::from(*self).checked_sub(size.into())?;
            let max_top_left = max_top_left.clamp_min((0, 0));
            let top_left = top_left.clamp((0, 0), max_top_left);

            let top_left_crop: Size = top_left.try_into()?;
            let bottom_right_crop = top_left_crop.checked_add(size)?;
            let bottom_right_crop = bottom_right_crop.clamp((0, 0), *self);
            let bottom_right_crop = self.checked_sub(bottom_right_crop)?;

            let crop_sides = Sides {
                top: top_left_crop.height,
                left: top_left_crop.width,
                bottom: bottom_right_crop.height,
                right: bottom_right_crop.width,
            };
            Ok::<Sides, arithmetic::Error>(crop_sides)
        })() {
            Ok(crop_sides) => Ok(crop_sides),
            Err(err) => Err(CropToFitError {
                size: *self,
                container: size,
                source: err.into(),
            }),
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
    type Error = CastError<super::Point, Size>;

    #[inline]
    fn try_from(point: super::Point) -> Result<Self, Self::Error> {
        match (|| {
            let width = point.x.cast::<u32>()?;
            let height = point.y.cast::<u32>()?;
            Ok::<Size, CastError<i64, u32>>(Self { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(CastError {
                src: point,
                target: std::marker::PhantomData,
                cause: Some(err.into()),
            }),
        }
    }
}

impl Clamp for Size {
    #[inline]
    fn clamp<MIN, MAX>(self, min: MIN, max: MAX) -> Self
    where
        MIN: Into<Self>,
        MAX: Into<Self>,
    {
        let min = min.into();
        let max = max.into();
        Self {
            width: num::clamp(self.width, min.width, max.width),
            height: num::clamp(self.height, min.height, max.height),
        }
    }
}

impl<F> CheckedMul<F> for Size
where
    F: arithmetic::Cast + arithmetic::Type,
{
    type Output = Self;
    type Error = ScaleByError;

    #[inline]
    fn checked_mul(self, scalar: F) -> Result<Self::Output, Self::Error> {
        self.scale_by::<_, Round>(scalar)
    }
}

impl<F> CheckedDiv<F> for Size
where
    F: arithmetic::Cast + arithmetic::Type + num::traits::Inv<Output = F>,
{
    type Output = Self;
    type Error = ScaleByError;

    #[inline]
    fn checked_div(self, scalar: F) -> Result<Self::Output, Self::Error> {
        let inverse = scalar.inv();
        self.scale_by::<_, Round>(inverse)
    }
}

impl CheckedSub<Sides> for Size {
    type Output = Self;
    type Error = ops::SubError<Self, Sides>;

    #[inline]
    fn checked_sub(self, rhs: Sides) -> Result<Self::Output, Self::Error> {
        use arithmetic::error::Operation;
        match (|| {
            let mut width = CheckedSub::checked_sub(self.width, rhs.left)?;
            width = CheckedSub::checked_sub(width, rhs.right)?;
            let mut height = CheckedSub::checked_sub(self.height, rhs.top)?;
            height = CheckedSub::checked_sub(height, rhs.bottom)?;
            Ok::<Self, ops::SubError<u32, u32>>(Self { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(ops::SubError(Operation {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err.into()),
            })),
        }
    }
}

impl CheckedAdd<Sides> for Size {
    type Output = Self;
    type Error = ops::AddError<Self, Sides>;

    #[inline]
    fn checked_add(self, rhs: Sides) -> Result<Self::Output, Self::Error> {
        use arithmetic::error::Operation;
        match (|| {
            let mut width = CheckedAdd::checked_add(self.width, rhs.left)?;
            width = CheckedAdd::checked_add(width, rhs.right)?;
            let mut height = CheckedAdd::checked_add(self.height, rhs.top)?;
            height = CheckedAdd::checked_add(height, rhs.bottom)?;
            Ok::<Self, ops::AddError<u32, u32>>(Self { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(ops::AddError(Operation {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err.into()),
            })),
        }
    }
}

impl CheckedAdd for Size {
    type Output = Self;
    type Error = ops::AddError<Self, Self>;

    #[inline]
    fn checked_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        use arithmetic::error::Operation;
        match (|| {
            let width = CheckedAdd::checked_add(self.width, rhs.width)?;
            let height = CheckedAdd::checked_add(self.height, rhs.height)?;
            Ok::<Self, ops::AddError<u32, u32>>(Self { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(ops::AddError(Operation {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err.into()),
            })),
        }
    }
}

impl CheckedSub for Size {
    type Output = Self;
    type Error = ops::SubError<Self, Self>;

    #[inline]
    fn checked_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        use arithmetic::error::Operation;
        match (|| {
            let width = CheckedSub::checked_sub(self.width, rhs.width)?;
            let height = CheckedSub::checked_sub(self.height, rhs.height)?;
            Ok::<Self, ops::SubError<u32, u32>>(Self { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(ops::SubError(Operation {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err.into()),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Size;
    use crate::arithmetic::{ops::CheckedAdd, Ceil, Floor, Round};
    use crate::error::Report;
    use crate::test::assert_matches_regex;
    use crate::types::{sides::abs::Sides, CropMode, Point, Rect};
    use pretty_assertions::assert_eq;

    #[test]
    fn crop_to_fit_equal_size() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let size = Size {
            width: 10,
            height: 10,
        };
        let expected = Sides {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        };
        assert_eq!(
            container.crop_to_fit(size, CropMode::Center).ok(),
            Some(expected),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Left).ok(),
            Some(expected),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Right).ok(),
            Some(expected),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Top).ok(),
            Some(expected),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Bottom).ok(),
            Some(expected),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: 10, y: 10 })
                .ok(),
            Some(expected),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: -10, y: -10 })
                .ok(),
            Some(expected),
        );
    }

    #[test]
    fn crop_to_fit_smaller_size() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let size = Size {
            width: 4,
            height: 4,
        };
        assert_eq!(
            container.crop_to_fit(size, CropMode::Center).ok(),
            Some(Sides {
                top: 3,
                left: 3,
                bottom: 3,
                right: 3,
            }),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Left).ok(),
            Some(Sides {
                top: 3,
                left: 0,
                bottom: 3,
                right: 6,
            }),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Right).ok(),
            Some(Sides {
                top: 3,
                left: 6,
                bottom: 3,
                right: 0,
            }),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Top).ok(),
            Some(Sides {
                top: 0,
                left: 3,
                bottom: 6,
                right: 3,
            }),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Bottom).ok(),
            Some(Sides {
                top: 6,
                left: 3,
                bottom: 0,
                right: 3,
            }),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: 2, y: 1 })
                .ok(),
            Some(Sides {
                top: 4,
                left: 5,
                bottom: 2,
                right: 1,
            }),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: -1, y: -2 })
                .ok(),
            Some(Sides {
                top: 1,
                left: 2,
                bottom: 5,
                right: 4,
            }),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: 100, y: 100 })
                .ok(),
            Some(Sides {
                top: 6,
                left: 6,
                bottom: 0,
                right: 0,
            }),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: -100, y: -100 })
                .ok(),
            Some(Sides {
                top: 0,
                left: 0,
                bottom: 6,
                right: 6,
            }),
        );
    }

    #[test]
    fn crop_to_fit_larger_size() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let size = Size {
            width: 20,
            height: 20,
        };
        let expected = Sides {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        };
        assert_eq!(
            container.crop_to_fit(size, CropMode::Center).ok(),
            Some(expected),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Left).ok(),
            Some(expected),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Right).ok(),
            Some(expected),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Top).ok(),
            Some(expected),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Bottom).ok(),
            Some(expected),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: 2, y: 1 })
                .ok(),
            Some(expected),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: -2, y: -1 })
                .ok(),
            Some(expected),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: 100, y: 100 })
                .ok(),
            Some(expected),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: -100, y: -100 })
                .ok(),
            Some(expected),
        );
    }

    #[test]
    fn crop_to_fit_zero_size() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let size = Size {
            width: 0,
            height: 0,
        };
        assert_eq!(
            container.crop_to_fit(size, CropMode::Center).ok(),
            Some(Sides {
                top: 5,
                left: 5,
                bottom: 5,
                right: 5,
            }),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Left).ok(),
            Some(Sides {
                top: 5,
                left: 0,
                bottom: 5,
                right: 10,
            }),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Right).ok(),
            Some(Sides {
                top: 5,
                left: 10,
                bottom: 5,
                right: 0,
            }),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Top).ok(),
            Some(Sides {
                top: 0,
                left: 5,
                bottom: 10,
                right: 5,
            }),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Bottom).ok(),
            Some(Sides {
                top: 10,
                left: 5,
                bottom: 0,
                right: 5,
            }),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: 2, y: 1 })
                .ok(),
            Some(Sides {
                top: 6,
                left: 7,
                bottom: 4,
                right: 3,
            }),
        );
        assert_eq!(
            container
                .crop_to_fit(size, CropMode::Custom { x: 100, y: 100 })
                .ok(),
            Some(Sides {
                top: 10,
                left: 10,
                bottom: 0,
                right: 0,
            }),
        );
        let container = Size {
            width: 0,
            height: 0,
        };
        let size = Size {
            width: 10,
            height: 10,
        };
        assert_eq!(
            container.crop_to_fit(size, CropMode::Center).ok(),
            Some(Sides {
                top: 0,
                left: 0,
                bottom: 0,
                right: 0,
            }),
        );
        assert_eq!(
            container.crop_to_fit(size, CropMode::Left).ok(),
            Some(Sides {
                top: 0,
                left: 0,
                bottom: 0,
                right: 0,
            }),
        );
    }

    #[test]
    fn center_equal_size() {
        assert_eq!(
            Size {
                width: 10,
                height: 10,
            }
            .center(Size {
                width: 10,
                height: 10
            })
            .ok(),
            Some(Rect::from_points(
                Point { x: 0, y: 0 },
                Point { x: 10, y: 10 }
            )),
        );
    }

    #[test]
    fn center_smaller_size() {
        assert_eq!(
            Size {
                width: 10,
                height: 10,
            }
            .center(Size {
                width: 4,
                height: 4
            })
            .ok(),
            Some(Rect::from_points(
                Point { x: 3, y: 3 },
                Point { x: 7, y: 7 }
            )),
        );
    }

    #[test]
    fn center_larger_size() {
        assert_eq!(
            Size {
                width: 10,
                height: 10,
            }
            .center(Size {
                width: 15,
                height: 15
            })
            .ok(),
            Some(Rect::from_points(
                Point { x: -3, y: -3 },
                Point { x: 12, y: 12 }
            )),
        );
    }

    #[test]
    fn center_zero_sizes() {
        assert_eq!(
            Size {
                width: 0,
                height: 0,
            }
            .center(Size {
                width: 15,
                height: 15
            })
            .ok(),
            Some(Rect::from_points(
                Point { x: -8, y: -8 },
                Point { x: 7, y: 7 }
            )),
            "center in zero size"
        );
        assert_eq!(
            Size {
                width: 10,
                height: 10,
            }
            .center(Size {
                width: 0,
                height: 0,
            })
            .ok(),
            Some(Rect::from_points(
                Point { x: 5, y: 5 },
                Point { x: 5, y: 5 }
            )),
            "center zero size in non-zero size"
        );
        assert_eq!(
            Size {
                width: 0,
                height: 0,
            }
            .center(Size {
                width: 0,
                height: 0,
            })
            .ok(),
            Some(Rect::from_points(
                Point { x: 0, y: 0 },
                Point { x: 0, y: 0 }
            )),
            "center zero size in zero size"
        );
    }
}
