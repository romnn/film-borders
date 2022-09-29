pub mod bounded_size;
pub mod color;
pub mod point;
pub mod rect;
pub mod rotation;
pub mod sides;
pub mod size;

pub use bounded_size::BoundedSize;
pub use color::Color;
pub use point::Point;
pub use rect::Rect;
pub use rotation::Rotation;
pub use sides::{Sides, SidesPercent};
pub use size::Size;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum CropMode {
    Custom { x: i64, y: i64 },
    Center,
    Bottom,
    Top,
    Left,
    Right,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ResizeMode {
    Fill,
    Cover,
    Contain,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum FitMode {
    Image,
    Border,
}

impl Default for FitMode {
    #[inline]
    fn default() -> Self {
        FitMode::Image
    }
}

impl std::str::FromStr for FitMode {
    type Err = super::error::ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "image" => Ok(FitMode::Image),
            "border" => Ok(FitMode::Border),
            _ => Err(super::error::ParseEnumError::Unknown(s.to_string())),
        }
    }
}
