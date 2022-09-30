use super::*;
use crate::error::*;
use crate::imageops::*;
use crate::numeric::ops::{self, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use crate::numeric::{self, error, Ceil, NumCast, Round};
use crate::{img, utils};
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

impl numeric::NumericType for Size {}

#[wasm_bindgen]
impl Size {
    #[wasm_bindgen(constructor)]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn wh(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub fn max(&self) -> u32 {
        max(self.width, self.height)
    }

    #[inline]
    pub fn min(&self) -> u32 {
        min(self.width, self.height)
    }
}

impl Size {
    #[inline]
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

    #[inline]
    pub fn center(self, size: Self) -> Rect {
        let container: Point = self.into();
        let size: Point = size.into();
        let top_left = container
            .checked_sub(size)
            .unwrap()
            .checked_div(2.0)
            .unwrap();
        let bottom_right = top_left.checked_add(size).unwrap();
        Rect {
            top: top_left.y,
            left: top_left.x,
            bottom: bottom_right.y,
            right: bottom_right.x,
        }
    }

    #[inline]
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

    #[inline]
    pub fn scale_by<F, R>(self, scalar: F) -> Result<Self, numeric::Error>
    where
        R: numeric::RoundingMode,
        F: numeric::NumCast + numeric::NumericType,
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

        // let scalar: f64 = num::NumCast::from(scalar).unwrap();
        // let width = max(R::round(self.width as f64 * scalar) as u64, 1);
        // let height = max(R::round(self.height as f64 * scalar) as u64, 1);
        // if width > u32::MAX as u64 {
        //     let ratio = u32::MAX as f64 / self.width as f64;
        //     let height = max((self.height as f64 * ratio).round() as u32, 1);
        //     Self {
        //         width: u32::MAX,
        //         height,
        //     }
        // } else if height > u32::MAX as u64 {
        //     let ratio = u32::MAX as f64 / self.height as f64;
        //     let width = max((self.width as f64 * ratio).round() as u32, 1);
        //     Self {
        //         width,
        //         height: u32::MAX,
        //     }
        // } else {
        //     Self {
        //         width: width as u32,
        //         height: height as u32,
        //     }
        // }
    }

    #[inline]
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

    #[inline]
    pub fn scale_to<S: Into<Size>>(self, container: S, mode: ResizeMode) -> Self {
        let container = container.into();
        match mode {
            ResizeMode::Fill => container,
            _ => {
                let scale = self.scale_factor(container, mode);
                self.scale_by::<_, Ceil>(scale.0).unwrap()
            }
        }
    }

    #[inline]
    pub fn crop_to_fit(&self, container: Size, mode: CropMode) -> Sides {
        // avoid underflow if container is larger than self
        let container = container.clamp((0, 0), *self);

        assert!(self.width >= container.width);
        assert!(self.height >= container.height);

        let center_top_left = self.center(container).top_left();

        let top_left: Point = match mode {
            CropMode::Custom { x, y } => center_top_left.checked_add(Point { x, y }).unwrap(),
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

        // Sides {
        //     top: top_left.height,
        //     left: top_left.width,
        //     bottom: bottom_right.height,
        //     right: bottom_right.width,
        // }
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

impl TryFrom<Point> for Size {
    type Error = numeric::CastError<Point, Size>;

    #[inline]
    fn try_from(point: Point) -> Result<Self, Self::Error> {
        match (|| {
            let width = point.x.cast::<u32>()?;
            let height = point.y.cast::<u32>()?;
            Ok::<Size, numeric::CastError<i64, u32>>(Self { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(numeric::CastError {
                src: point,
                target: std::marker::PhantomData,
                cause: Some(Box::new(err)),
            }),
        }
    }
}

impl<F> CheckedMul<F> for Size
where
    F: numeric::NumCast + numeric::NumericType,
{
    type Output = Self;
    type Error = ops::MulError<Self, F>;

    #[inline]
    fn checked_mul(self, scalar: F) -> Result<Self::Output, Self::Error> {
        match self.scale_by::<_, Round>(scalar) {
            Ok(size) => Ok(size),
            Err(numeric::Error(err)) => Err(ops::MulError(error::ArithmeticError {
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
    F: numeric::NumCast + numeric::NumericType + num::traits::Inv<Output = F>,
{
    type Output = Self;
    type Error = ops::DivError<Self, F>;

    #[inline]
    fn checked_div(self, scalar: F) -> Result<Self::Output, Self::Error> {
        use num::traits::Inv;
        let inverse = scalar.inv();
        match self.scale_by::<_, Round>(inverse) {
            Ok(size) => Ok(size),
            Err(numeric::Error(err)) => Err(ops::DivError(error::ArithmeticError {
                lhs: self,
                rhs: inverse,
                kind: None,
                cause: Some(err),
            })),
        }
    }
}

// impl<F> std::ops::Mul<F> for Size
// where
//     F: num::NumCast,
// {
//     type Output = Self;

//     #[inline]
//     fn mul(self, scalar: F) -> Self::Output {
//         self.scale_by::<_, Round>(scalar).unwrap()
//     }
// }

// impl<F> std::ops::Div<F> for Size
// where
//     F: num::NumCast,
// {
//     type Output = Self;

//     #[inline]
//     fn div(self, scalar: F) -> Self::Output {
//         let scalar: f64 = num::NumCast::from(scalar).unwrap();
//         self.scale_by::<_, Round>(1.0 / scalar).unwrap()
//     }
// }

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
            Err(err) => Err(ops::SubError(numeric::ArithmeticError {
                lhs: self,
                rhs: rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}

// impl std::ops::Sub<Sides> for Size {
//     type Output = Self;

//     #[inline]
//     fn sub(self, sides: Sides) -> Self::Output {
//         let width = self.width as i64 - sides.width() as i64;
//         let height = self.height as i64 - sides.height() as i64;
//         Size {
//             width: utils::clamp(width, 0, u32::MAX as i64) as u32,
//             height: utils::clamp(height, 0, u32::MAX as i64) as u32,
//         }
//     }
// }

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
            Err(err) => Err(ops::AddError(numeric::ArithmeticError {
                lhs: self,
                rhs: rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}

// impl std::ops::Add<Sides> for Size {
//     type Output = Self;

//     #[inline]
//     fn add(self, sides: Sides) -> Self::Output {
//         let width = self.width as i64 + sides.width() as i64;
//         let height = self.height as i64 + sides.height() as i64;
//         Size {
//             width: utils::clamp(width, 0, u32::MAX as i64) as u32,
//             height: utils::clamp(height, 0, u32::MAX as i64) as u32,
//         }
//     }
// }

// impl std::ops::Add<Point> for Size {
//     type Output = Self;

//     #[inline]
//     fn add(self, p: Point) -> Self::Output {
//         let width = utils::clamp(self.width as i64 + p.x, 0, u32::MAX as i64);
//         let height = utils::clamp(self.height as i64 + p.y, 0, u32::MAX as i64);
//         Size {
//             width: width as u32,
//             height: height as u32,
//         }
//     }
// }

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
            Err(err) => Err(ops::AddError(error::ArithmeticError {
                lhs: self,
                rhs: rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}

// impl std::ops::Add for Size {
//     type Output = Self;

//     #[inline]
//     fn add(self, rhs: Self) -> Self::Output {
//         Size {
//             width: self.width + rhs.width,
//             height: self.height + rhs.height,
//         }
//     }
// }

// impl CheckedAdd<Sides> for Size {
//     type Output = Self;

//     #[inline]
//     fn checked_add(self, rhs: Sides) -> Result<Self::Output, ops::AddError<Self, Sides>> {
//         match (|| {
//             let width = CheckedAdd::checked_add(self.width, rhs.width())?;
//             let height = CheckedAdd::checked_add(self.height, rhs.height())?;
//             Ok(Self { width, height })
//         })() {
//             Ok(size) => Ok(size),
//             Err(ops::AddError(err)) => Err(ops::AddError(numeric::ArithmeticError {
//                 lhs: self,
//                 rhs: rhs,
//                 container_type_name: err.container_type_name,
//                 kind: err.kind,
//             })),
//         }
//     }
// }

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
            Err(err) => Err(ops::SubError(error::ArithmeticError {
                lhs: self,
                rhs: rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}

// impl std::ops::Sub for Size {
//     type Output = Self;

//     #[inline]
//     fn sub(self, rhs: Self) -> Self::Output {
//         Size {
//             width: self.width - rhs.width,
//             height: self.height - rhs.height,
//         }
//     }
// }
