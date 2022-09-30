use super::*;
use crate::error::*;
use crate::imageops::*;
use crate::numeric::ops::{CheckedAdd, CheckedDiv, CheckedSub};
use crate::numeric::{Ceil, Round, RoundingMode};
use crate::{img, utils};
use num::traits::NumCast;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::path::Path;
use wasm_bindgen::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Rect {
    pub top: i64,
    pub left: i64,
    pub right: i64,
    pub bottom: i64,
}

impl Rect {
    pub fn new(top_left: Point, size: Size) -> Self {
        let bottom_right = top_left.checked_add(Point::from(size)).unwrap();
        Self {
            top: top_left.y,
            left: top_left.x,
            bottom: bottom_right.y,
            right: bottom_right.x,
        }
    }

    pub fn from_points(p1: Point, p2: Point) -> Self {
        Self {
            top: min(p1.y, p2.y),
            bottom: max(p1.y, p2.y),
            left: min(p1.x, p2.x),
            right: max(p1.x, p2.x),
        }
    }

    #[inline]
    pub fn pixel_count(&self) -> u64 {
        self.width() as u64 * self.height() as u64
    }

    #[inline]
    pub fn size(&self) -> Size {
        let size = self.bottom_right().checked_sub(self.top_left()).unwrap();
        size.try_into().unwrap()
    }

    #[inline]
    pub fn center(&self) -> Point {
        self.top_left()
            .checked_add(Point::from(self.size()).checked_div(2.0).unwrap())
            .unwrap()
    }

    #[inline]
    pub fn crop_mode(&self, container: &Rect) -> CropMode {
        let offset = container.center().checked_sub(self.center()).unwrap();
        CropMode::Custom {
            x: offset.x,
            y: offset.y,
        }
    }

    #[inline]
    pub fn top_right(&self) -> Point {
        Point {
            y: self.top,
            x: self.right,
        }
    }

    #[inline]
    pub fn top_left(&self) -> Point {
        Point {
            y: self.top,
            x: self.left,
        }
    }

    #[inline]
    pub fn bottom_left(&self) -> Point {
        Point {
            y: self.bottom,
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
