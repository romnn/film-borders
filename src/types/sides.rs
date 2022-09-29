#[cfg(feature = "borders")]
use crate::borders;
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
use super::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct SidesPercent {
    pub top: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
}

fn percent_to_abs(percent: f32, dimension: u32) -> u32 {
    let percent = percent.max(0.0);
    if percent <= 1.0 {
        let absolute = percent * dimension as f32;
        utils::clamp(absolute, 0.0, dimension as f32).ceil() as u32
    } else {
        utils::clamp(percent, 0.0, dimension as f32).ceil() as u32
    }
}

impl std::ops::Mul<u32> for SidesPercent {
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

impl<F> std::ops::Mul<F> for Sides
where
    F: NumCast,
{
    type Output = Self;

    fn mul(self, scalar: F) -> Self::Output {
        let scalar: f32 = NumCast::from(scalar).unwrap();
        Self {
            top: (self.top as f32 * scalar).ceil() as u32,
            right: (self.right as f32 * scalar).ceil() as u32,
            bottom: (self.bottom as f32 * scalar).ceil() as u32,
            left: (self.left as f32 * scalar).ceil() as u32,
        }
    }
}
