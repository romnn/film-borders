use super::{sides::abs::Sides, Point, Size};
use crate::arithmetic::{
    self,
    ops::{self, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
    Cast, CastError,
};

struct Bounds {
    x: std::ops::RangeInclusive<i64>,
    y: std::ops::RangeInclusive<i64>,
}

mod sealed {
    use crate::arithmetic::{
        self,
        ops::{self, CheckedAdd},
    };
    use crate::error;
    use crate::types::{Point, Size};

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub struct Rect {
        pub top: i64,
        pub left: i64,
        pub bottom: i64,
        pub right: i64,
        _sealed: (),
    }

    impl arithmetic::Type for Rect {}

    impl Rect {
        #[inline]
        pub fn new(
            top_left: impl Into<Point>,
            size: impl Into<Size>,
        ) -> Result<Self, super::Error> {
            let top_left = top_left.into();
            let size = size.into();
            let bottom_right =
                top_left
                    .checked_add(Point::from(size))
                    .map_err(|err| super::Error {
                        top_left,
                        size,
                        source: err,
                    })?;
            Ok(Self {
                top: top_left.y,
                left: top_left.x,
                bottom: bottom_right.y,
                right: bottom_right.x,
                _sealed: (),
            })
        }

        #[inline]
        #[must_use]
        pub fn from_points(p1: impl Into<Point>, p2: impl Into<Point>) -> Self {
            let p1 = p1.into();
            let p2 = p2.into();
            Self {
                top: p1.y.min(p2.y),
                bottom: p1.y.max(p2.y),
                left: p1.x.min(p2.x),
                right: p1.x.max(p2.x),
                _sealed: (),
            }
        }

        #[inline]
        #[must_use]
        pub fn clamp(self, bounds: &Self) -> Self {
            let top = self.top.clamp(bounds.top, bounds.bottom);
            let left = self.left.clamp(bounds.left, bounds.right);
            let bottom = self.bottom.clamp(bounds.top, bounds.bottom);
            let right = self.right.clamp(bounds.left, bounds.right);
            Self {
                top,
                left,
                bottom,
                right,
                _sealed: (),
            }
        }
    }

    impl CheckedAdd<Point> for Rect {
        type Output = Self;
        type Error = ops::AddError<Self, Point>;

        #[inline]
        fn checked_add(self, rhs: Point) -> Result<Self::Output, Self::Error> {
            use arithmetic::error::Operation;
            match (|| {
                let top = CheckedAdd::checked_add(self.top, rhs.y)?;
                let left = CheckedAdd::checked_add(self.left, rhs.x)?;
                let bottom = CheckedAdd::checked_add(self.bottom, rhs.y)?;
                let right = CheckedAdd::checked_add(self.right, rhs.x)?;
                Ok::<Self, ops::AddError<i64, i64>>(Self {
                    top,
                    left,
                    bottom,
                    right,
                    _sealed: (),
                })
            })() {
                Ok(rect) => Ok(rect),
                Err(err) => Err(ops::AddError(Operation {
                    lhs: self,
                    rhs,
                    kind: None,
                    cause: Some(err.into()),
                })),
            }
        }
    }
}

pub use sealed::Rect;

impl Rect {
    #[inline]
    #[must_use]
    pub fn width(&self) -> i64 {
        // safety: this is safe because this invariant holds
        // left <= right
        self.right - self.left
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> i64 {
        // safety: this is safe because this invariant holds
        // top <= bottom
        self.bottom - self.top
    }

    #[inline]
    #[must_use]
    pub fn top_right(&self) -> Point {
        Point {
            y: self.top,
            x: self.right,
        }
    }

    #[inline]
    #[must_use]
    pub fn top_left(&self) -> Point {
        Point {
            y: self.top,
            x: self.left,
        }
    }

    #[inline]
    #[must_use]
    pub fn bottom_left(&self) -> Point {
        Point {
            y: self.bottom,
            x: self.left,
        }
    }

    #[inline]
    #[must_use]
    pub fn bottom_right(&self) -> Point {
        Point {
            y: self.bottom,
            x: self.right,
        }
    }

    #[inline]
    pub fn pixel_count(&self) -> Result<u64, PixelCountError> {
        match (|| {
            let width = self.width().cast::<u64>()?;
            let height = self.height().cast::<u64>()?;
            let pixel_count = CheckedMul::checked_mul(width, height)?;
            Ok::<u64, arithmetic::Error>(pixel_count)
        })() {
            Ok(pixel_count) => Ok(pixel_count),
            Err(err) => Err(PixelCountError {
                rect: *self,
                source: err.into(),
            }),
        }
    }

    #[inline]
    pub fn size(&self) -> Result<Size, SizeError> {
        match (|| {
            let width = self.width().cast::<u32>()?;
            let height = self.height().cast::<u32>()?;
            Ok::<Size, CastError<i64, u32>>(Size { width, height })
        })() {
            Ok(size) => Ok(size),
            Err(err) => Err(SizeError {
                rect: *self,
                source: err.into(),
            }),
        }
    }

    #[inline]
    pub fn center(&self) -> Result<Point, CenterError> {
        let size = Point {
            x: self.width(),
            y: self.height(),
        };
        match (|| {
            let rel_center = size.checked_div(2.0)?;
            let center = self.top_left().checked_add(rel_center)?;
            Ok::<Point, arithmetic::Error>(center)
        })() {
            Ok(center) => Ok(center),
            Err(err) => Err(CenterError {
                rect: *self,
                source: err,
            }),
        }
    }

    #[inline]
    pub fn center_offset_to(&self, parent: &Rect) -> Result<Point, CenterOffsetError> {
        match (|| {
            let child_center = self.center()?;
            let parent_center = parent.center()?;
            let offset = parent_center.checked_sub(child_center)?;
            Ok::<Point, CenterOffsetErrorSource>(offset)
        })() {
            Ok(center) => Ok(center),
            Err(err) => Err(CenterOffsetError {
                parent: *parent,
                child: *self,
                source: err.into(),
            }),
        }
    }

    #[inline]
    pub fn padded(self, padding: u32) -> Result<Self, PadError> {
        self.checked_add(Sides::uniform(padding))
            .map_err(|err| PadError {
                rect: self,
                padding,
                source: err,
            })
    }

    #[inline]
    pub fn contains(&self, point: &Point) -> bool {
        let bounds = Bounds {
            x: self.left..=self.right,
            y: self.top..=self.bottom,
        };
        bounds.x.contains(&point.x) && bounds.y.contains(&point.y)
    }

    #[inline]
    pub fn has_intersection(&self, other: &Self) -> bool {
        let self_intersects_other = self.intersects(other);
        let other_intersects_self = other.intersects(self);
        self_intersects_other || other_intersects_self
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        let contains_tl = self.contains(&other.top_left());
        let contains_br = self.contains(&other.bottom_right());
        contains_tl || contains_br
    }

    #[inline]
    pub fn extend_to(&mut self, point: &Point) {
        self.top = self.top.min(point.y);
        self.left = self.left.min(point.x);
        self.bottom = self.bottom.max(point.y);
        self.right = self.right.max(point.x);
    }
}

impl From<Size> for Rect {
    fn from(size: Size) -> Self {
        Self::from_points((0, 0), size)
    }
}

impl From<Point> for Rect {
    fn from(point: Point) -> Self {
        Self::from_points(point, point)
    }
}

impl std::fmt::Debug for Rect {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Rect")
            .field("top", &self.top)
            .field("left", &self.left)
            .field("bottom", &self.bottom)
            .field("right", &self.right)
            .field("size", &self.size().ok())
            .field("center", &self.center().ok())
            .field("pixel_count", &self.pixel_count().ok())
            .finish()
    }
}

impl std::fmt::Display for Rect {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Rect")
            .field("top", &self.top)
            .field("left", &self.left)
            .field("bottom", &self.bottom)
            .field("right", &self.right)
            .finish()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SubSidesError {
    #[error("subtracting {sides} from {rect} exceeds bounds")]
    OutOfBounds { sides: Sides, rect: Rect },
    #[error(transparent)]
    Arithmetic(#[from] arithmetic::Error),
}

impl CheckedSub<Sides> for Rect {
    type Output = Self;
    type Error = SubSidesError;

    #[inline]
    fn checked_sub(self, rhs: Sides) -> Result<Self::Output, Self::Error> {
        use arithmetic::error::Operation;
        match (|| {
            let sides_top = i64::from(rhs.top);
            let sides_left = i64::from(rhs.left);
            let sides_bottom = i64::from(rhs.bottom);
            let sides_right = i64::from(rhs.right);
            let top = CheckedAdd::checked_add(self.top, sides_top)?;
            let left = CheckedAdd::checked_add(self.left, sides_left)?;
            let bottom = CheckedSub::checked_sub(self.bottom, sides_bottom)?;
            let right = CheckedSub::checked_sub(self.right, sides_right)?;

            let top_left = Point { x: left, y: top };
            let bottom_right = Point {
                x: right,
                y: bottom,
            };
            Ok::<_, arithmetic::Error>((top_left, bottom_right))
        })() {
            Ok((top_left, bottom_right)) => {
                // check if invariants were violated
                if top_left.x > bottom_right.x || top_left.y > bottom_right.y {
                    Err(SubSidesError::OutOfBounds {
                        sides: rhs,
                        rect: self,
                    })
                } else {
                    let rect = Self::from_points(top_left, bottom_right);
                    Ok(rect)
                }
            }
            Err(err) => Err(SubSidesError::Arithmetic(arithmetic::Error::from(
                ops::SubError(Operation {
                    lhs: self,
                    rhs,
                    kind: None,
                    cause: Some(err),
                }),
            ))),
        }
    }
}

impl CheckedAdd<Sides> for Rect {
    type Output = Self;
    type Error = ops::AddError<Self, Sides>;

    #[inline]
    fn checked_add(self, rhs: Sides) -> Result<Self::Output, Self::Error> {
        use arithmetic::error::Operation;
        match (|| {
            let sides_top = i64::from(rhs.top);
            let sides_left = i64::from(rhs.left);
            let sides_bottom = i64::from(rhs.bottom);
            let sides_right = i64::from(rhs.right);
            let top = CheckedSub::checked_sub(self.top, sides_top)?;
            let left = CheckedSub::checked_sub(self.left, sides_left)?;
            let bottom = CheckedAdd::checked_add(self.bottom, sides_bottom)?;
            let right = CheckedAdd::checked_add(self.right, sides_right)?;
            let top_left = Point { x: left, y: top };
            let bottom_right = Point {
                x: right,
                y: bottom,
            };
            let rect = Self::from_points(top_left, bottom_right);
            Ok::<Self, arithmetic::Error>(rect)
        })() {
            Ok(rect) => Ok(rect),
            Err(err) => Err(ops::AddError(Operation {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err),
            })),
        }
    }
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to create rect at {top_left} of size {size}")]
pub struct Error {
    top_left: Point,
    size: Size,
    source: ops::AddError<Point, Point>,
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to compute pixel count for {rect}")]
pub struct PixelCountError {
    rect: Rect,
    source: arithmetic::Error,
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to compute center point of {rect}")]
pub struct CenterError {
    rect: Rect,
    source: arithmetic::Error,
}

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum CenterOffsetErrorSource {
    #[error(transparent)]
    Center(#[from] CenterError),

    #[error(transparent)]
    Arithmetic(#[from] ops::SubError<Point, Point>),
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to compute center offset from {parent} to {child}")]
pub struct CenterOffsetError {
    parent: Rect,
    child: Rect,
    source: CenterOffsetErrorSource,
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to compute size for {rect}")]
pub struct SizeError {
    rect: Rect,
    source: CastError<i64, u32>,
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to add padding of {padding} to {rect}")]
pub struct PadError {
    rect: Rect,
    padding: u32,
    source: ops::AddError<Rect, Sides>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pixel_count() {
        use crate::types::Size;
        let size = Size {
            width: 10,
            height: 10,
        };
        assert_eq!(
            Rect::from_points((0, 0), size).pixel_count().ok(),
            Some(100)
        );

        assert!(u64::from(u32::MAX)
            .checked_mul(u64::from(u32::MAX))
            .is_some());
    }
}
