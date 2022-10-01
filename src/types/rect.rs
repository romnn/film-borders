use super::sides::abs::Sides;
use super::{CropMode, Point, Size};
use crate::arithmetic;
use crate::arithmetic::ops::{self, CheckedAdd, CheckedDiv, CheckedSub};
use crate::Error;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Rect {
    pub top: i64,
    pub left: i64,
    pub bottom: i64,
    pub right: i64,
}

impl arithmetic::Type for Rect {}

impl std::fmt::Debug for Rect {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Rect")
            .field("top", &self.top)
            .field("left", &self.left)
            .field("bottom", &self.bottom)
            .field("right", &self.right)
            .field("size", &self.size().ok())
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

impl Rect {
    #[inline]
    pub fn new(top_left: Point, size: Size) -> Result<Self, Error> {
        let bottom_right =
            top_left
                .checked_add(Point::from(size))
                .map_err(|err| Error::Arithmetic {
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
    pub fn from_points(p1: Point, p2: Point) -> Self {
        Self {
            top: p1.y.min(p2.y),
            bottom: p1.y.max(p2.y),
            left: p1.x.min(p2.x),
            right: p1.x.max(p2.x),
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
    pub fn size(&self) -> Result<Size, Error> {
        let top_left = self.top_left();
        let bottom_right = self.bottom_right();
        let size = bottom_right
            .checked_sub(top_left)
            .map_err(|err| Error::Arithmetic {
                msg: format!(
                    "failed to compute size from top right {} and bottom left {}",
                    top_left, bottom_right
                ),
                source: err.into(),
            })?;
        size.try_into()
    }

    #[inline]
    pub fn center(&self) -> Result<Point, Error> {
        let size = self.size()?;
        let rel_center = Point::from(size)
            .checked_div(2.0)
            .map_err(arithmetic::Error::from);
        // .map_err(|err| Error::Arithmetic {
        //     msg: format!("failed to compute center point of size {}", size),
        //     source: err.into(),
        // });
        rel_center
            .and_then(|rel_center| {
                self.top_left()
                    .checked_add(rel_center)
                    .map_err(arithmetic::Error::from)
            })
            .map_err(|err| Error::Arithmetic {
                msg: format!("failed to compute center point of size {}", size),
                source: err,
            })
    }

    #[inline]
    #[must_use]
    pub fn crop_mode(&self, container: &Rect) -> CropMode {
        let self_center = self.center().unwrap();
        let container_center = container.center().unwrap();
        let offset = container_center.checked_sub(self_center).unwrap();
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
        self.size().unwrap().width
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u32 {
        self.size().unwrap().height
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
        self.top = self.top.min(point.y);
        self.left = self.left.min(point.x);
        self.bottom = self.bottom.max(point.y);
        self.right = self.right.max(point.x);
    }

    #[inline]
    #[must_use]
    pub fn extend(self, value: u32) -> Self {
        self.checked_add(Sides::uniform(value)).unwrap()
    }

    #[inline]
    #[must_use]
    pub fn contains(&self, x: i64, y: i64, padding: i64) -> bool {
        let y_top = self.top.checked_sub(padding).unwrap();
        let x_left = self.left.checked_sub(padding).unwrap();
        let y_bottom = self.bottom.checked_add(padding).unwrap();
        let x_right = self.right.checked_add(padding).unwrap();

        x_left <= x && x <= x_right && y_top <= y && y <= y_bottom
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
            Err(err) => Err(ops::AddError(arithmetic::error::Arithmetic {
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
            Err(arithmetic::Error(err)) => Err(ops::SubError(arithmetic::error::Arithmetic {
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
            Err(arithmetic::Error(err)) => Err(ops::AddError(arithmetic::error::Arithmetic {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err),
            })),
        }
    }
}
