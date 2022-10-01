use super::{sides::abs::Sides, Point, Size};
use crate::arithmetic::{
    self,
    ops::{self, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
};
use crate::error;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Rect {
    pub top: i64,
    pub left: i64,
    pub bottom: i64,
    pub right: i64,
}

impl arithmetic::Type for Rect {}

impl Rect {
    #[inline]
    pub fn new(
        top_left: impl Into<Point>,
        size: impl Into<Size>,
    ) -> Result<Self, error::Arithmetic> {
        let top_left = top_left.into();
        let size = size.into();
        let bottom_right =
            top_left
                .checked_add(Point::from(size))
                .map_err(|err| error::Arithmetic {
                    msg: format!("failed to create rect at {} of size {}", top_left, size),
                    source: err.into(),
                })?;
        Ok(Self {
            top: top_left.y,
            left: top_left.x,
            bottom: bottom_right.y,
            right: bottom_right.x,
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
        }
    }

    #[inline]
    pub fn pixel_count(&self) -> Result<u64, error::Arithmetic> {
        let size = self.size()?;
        let width = u64::from(size.width);
        let height = u64::from(size.height);

        CheckedMul::checked_mul(width, height).map_err(|err| error::Arithmetic {
            msg: format!("failed to compute pixel count for size {}", size),
            source: err.into(),
        })
    }

    #[inline]
    pub fn size(&self) -> Result<Size, error::Arithmetic> {
        let top_left = self.top_left();
        let bottom_right = self.bottom_right();
        let size = bottom_right
            .checked_sub(top_left)
            .map_err(|err| error::Arithmetic {
                msg: format!(
                    "failed to compute size from top right {} and bottom left {}",
                    top_left, bottom_right
                ),
                source: err.into(),
            })?;
        size.try_into()
    }

    #[inline]
    pub fn center(&self) -> Result<Point, error::Arithmetic> {
        let size = self.size()?;
        let rel_center = Point::from(size)
            .checked_div(2.0)
            .map_err(arithmetic::Error::from);
        rel_center
            .and_then(|rel_center| {
                self.top_left()
                    .checked_add(rel_center)
                    .map_err(arithmetic::Error::from)
            })
            .map_err(|err| error::Arithmetic {
                msg: format!("failed to compute center point of size {}", size),
                source: err,
            })
    }

    #[inline]
    pub fn center_offset_to(&self, container: &Rect) -> Result<Point, error::Arithmetic> {
        match (|| {
            let self_center = self.center()?;
            let container_center = container.center()?;
            let offset = container_center.checked_sub(self_center)?;
            Ok::<Point, arithmetic::Error>(offset)
        })() {
            Ok(center) => Ok(center),
            Err(err) => Err(error::Arithmetic {
                msg: format!(
                    "failed to compute center offset from {} to {}",
                    container, self
                ),
                source: err,
            }),
        }
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
    pub fn has_intersection(&self, other: &Self, padding: i64) -> Result<bool, error::Arithmetic> {
        let self_intersects_other = self.intersects(other, padding)?;
        let other_intersects_self = other.intersects(self, padding)?;
        Ok(self_intersects_other || other_intersects_self)
    }

    #[inline]
    pub fn intersects(&self, other: &Self, padding: i64) -> Result<bool, error::Arithmetic> {
        let contains_tl = self.contains(&other.top_left(), padding)?;
        let contains_br = self.contains(&other.bottom_right(), padding)?;
        Ok(contains_tl || contains_br)
    }

    #[inline]
    pub fn extend_to(&mut self, point: &Point) {
        self.top = self.top.min(point.y);
        self.left = self.left.min(point.x);
        self.bottom = self.bottom.max(point.y);
        self.right = self.right.max(point.x);
    }

    #[inline]
    pub fn extend(self, value: u32) -> Result<Self, error::Arithmetic> {
        self.checked_add(Sides::uniform(value))
            .map_err(|err| error::Arithmetic {
                msg: format!("failed to extend {} by {}", self, value),
                source: err.into(),
            })
    }

    #[inline]
    pub fn contains(&self, point: &Point, padding: i64) -> Result<bool, error::Arithmetic> {
        struct Bounds {
            x: std::ops::RangeInclusive<i64>,
            y: std::ops::RangeInclusive<i64>,
        }

        let bounds = (|| {
            let y_top = CheckedSub::checked_sub(self.top, padding)?;
            let x_left = CheckedSub::checked_sub(self.left, padding)?;
            let y_bottom = CheckedAdd::checked_add(self.bottom, padding)?;
            let x_right = CheckedAdd::checked_add(self.right, padding)?;
            Ok::<Bounds, arithmetic::Error>(Bounds {
                x: x_left..=x_right,
                y: y_top..=y_bottom,
            })
        })();

        let bounds = bounds.map_err(|err| error::Arithmetic {
            msg: format!("failed to add padding of {} to {}", padding, self),
            source: err,
        })?;

        Ok(bounds.x.contains(&point.x) && bounds.y.contains(&point.y))
    }

    #[inline]
    #[must_use]
    pub fn clip_to(self, bounds: &Self) -> Self {
        let top = self.top.clamp(bounds.top, bounds.bottom);
        let left = self.left.clamp(bounds.left, bounds.right);
        let bottom = self.bottom.clamp(bounds.top, bounds.bottom);
        let right = self.right.clamp(bounds.left, bounds.right);
        Self {
            top,
            left,
            bottom,
            right,
        }
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

impl CheckedAdd<Point> for Rect {
    type Output = Self;
    type Error = ops::AddError<Self, Point>;

    #[inline]
    fn checked_add(self, rhs: Point) -> Result<Self::Output, Self::Error> {
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
            })
        })() {
            Ok(rect) => Ok(rect),
            Err(err) => Err(ops::AddError(arithmetic::error::Operation {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}

impl CheckedSub<Sides> for Rect {
    type Output = Self;
    type Error = ops::SubError<Self, Sides>;

    #[inline]
    fn checked_sub(self, rhs: Sides) -> Result<Self::Output, Self::Error> {
        match (|| {
            let top = CheckedAdd::checked_add(self.top, i64::from(rhs.top))?;
            let left = CheckedAdd::checked_add(self.left, i64::from(rhs.left))?;
            let bottom = CheckedSub::checked_sub(self.bottom, i64::from(rhs.bottom))?;
            let right = CheckedSub::checked_sub(self.right, i64::from(rhs.right))?;
            Ok::<Self, arithmetic::Error>(Self {
                top,
                left,
                bottom,
                right,
            })
        })() {
            Ok(rect) => Ok(rect),
            Err(arithmetic::Error(err)) => Err(ops::SubError(arithmetic::error::Operation {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err),
            })),
        }
    }
}

impl CheckedAdd<Sides> for Rect {
    type Output = Self;
    type Error = ops::AddError<Self, Sides>;

    #[inline]
    fn checked_add(self, rhs: Sides) -> Result<Self::Output, Self::Error> {
        match (|| {
            let top = CheckedSub::checked_sub(self.top, i64::from(rhs.top))?;
            let left = CheckedSub::checked_sub(self.left, i64::from(rhs.left))?;
            let bottom = CheckedAdd::checked_add(self.bottom, i64::from(rhs.bottom))?;
            let right = CheckedAdd::checked_add(self.right, i64::from(rhs.right))?;
            Ok::<Self, arithmetic::Error>(Self {
                top,
                left,
                bottom,
                right,
            })
        })() {
            Ok(rect) => Ok(rect),
            Err(arithmetic::Error(err)) => Err(ops::AddError(arithmetic::error::Operation {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err),
            })),
        }
    }
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
