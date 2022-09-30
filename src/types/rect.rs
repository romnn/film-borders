use super::sides::abs::Sides;
use super::{CropMode, Point, Size};
use crate::numeric::ops::{self, CheckedAdd, CheckedDiv, CheckedSub};
use crate::numeric::{self, error};
use crate::utils;
use std::cmp::{max, min};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Rect {
    pub top: i64,
    pub left: i64,
    pub bottom: i64,
    pub right: i64,
}

impl Rect {
    #[inline]
    #[must_use]
    pub fn new(top_left: Point, size: Size) -> Self {
        let bottom_right = top_left.checked_add(Point::from(size)).unwrap();
        Self {
            top: top_left.y,
            left: top_left.x,
            bottom: bottom_right.y,
            right: bottom_right.x,
        }
    }

    #[inline]
    #[must_use]
    pub fn from_points(p1: Point, p2: Point) -> Self {
        Self {
            top: min(p1.y, p2.y),
            bottom: max(p1.y, p2.y),
            left: min(p1.x, p2.x),
            right: max(p1.x, p2.x),
        }
    }

    #[inline]
    #[must_use]
    pub fn pixel_count(&self) -> u64 {
        u64::from(self.width())
            .checked_mul(u64::from(self.height()))
            .unwrap()
    }

    #[inline]
    #[must_use]
    pub fn size(&self) -> Size {
        let size = self.bottom_right().checked_sub(self.top_left()).unwrap();
        size.try_into().unwrap()
    }

    #[inline]
    #[must_use]
    pub fn center(&self) -> Point {
        self.top_left()
            .checked_add(Point::from(self.size()).checked_div(2.0).unwrap())
            .unwrap()
    }

    #[inline]
    #[must_use]
    pub fn crop_mode(&self, container: &Rect) -> CropMode {
        let offset = container.center().checked_sub(self.center()).unwrap();
        CropMode::Custom {
            x: offset.x,
            y: offset.y,
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
    #[must_use]
    pub fn width(&self) -> u32 {
        self.size().width
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u32 {
        self.size().height
    }

    #[inline]
    #[must_use]
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
    #[must_use]
    pub fn extend(self, value: u32) -> Self {
        self.checked_add(Sides::uniform(value)).unwrap()
    }

    #[inline]
    #[must_use]
    pub fn contains(&self, x: i64, y: i64, padding: i64) -> bool {
        let y_top = self.top - padding;
        let x_left = self.left - padding;
        let y_bottom = self.bottom + padding;
        let x_right = self.right + padding;

        x_left <= x && x <= x_right && y_top <= y && y <= y_bottom
    }

    #[inline]
    #[must_use]
    pub fn clip_to(self, bounds: &Self) -> Self {
        let top = utils::clamp(self.top, bounds.top, bounds.bottom);
        let left = utils::clamp(self.left, bounds.left, bounds.right);
        let bottom = utils::clamp(self.bottom, bounds.top, bounds.bottom);
        let right = utils::clamp(self.right, bounds.left, bounds.right);
        Self {
            top,
            left,
            bottom,
            right,
        }
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
            Err(err) => Err(ops::AddError(error::ArithmeticError {
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
            Ok::<Self, numeric::Error>(Self {
                top,
                left,
                bottom,
                right,
            })
        })() {
            Ok(rect) => Ok(rect),
            Err(numeric::Error(err)) => Err(ops::SubError(error::ArithmeticError {
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
            Ok::<Self, numeric::Error>(Self {
                top,
                left,
                bottom,
                right,
            })
        })() {
            Ok(rect) => Ok(rect),
            Err(numeric::Error(err)) => Err(ops::AddError(error::ArithmeticError {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err),
            })),
        }
    }
}
