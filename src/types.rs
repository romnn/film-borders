#[cfg(feature = "borders")]
use super::borders;
use super::error;
use super::{img, Error};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub enum Border {
    #[cfg(feature = "borders")]
    Builtin(borders::BuiltinBorder),
    Custom(img::Image),
}

impl Border {
    #[inline]
    pub fn new<R: std::io::BufRead + std::io::Seek>(reader: R) -> Result<Self, Error> {
        Ok(Self::Custom(img::Image::new(reader)?))
    }

    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self::Custom(img::Image::open(path)?))
    }

    #[inline]
    pub fn from_image(img: img::Image) -> Self {
        Self::Custom(img)
    }

    #[inline]
    pub fn into_image(self) -> Result<img::Image, Error> {
        match self {
            #[cfg(feature = "borders")]
            Self::Builtin(builtin) => builtin.into_image(),
            Self::Custom(img) => Ok(img),
        }
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct OutputSize {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[wasm_bindgen]
impl OutputSize {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        OutputSize::default()
    }
}

macro_rules! from_hex {
    ($value:expr) => {
        $value
            .ok_or(error::ColorError::MissingComponent)
            .and_then(|v| {
                u8::from_str_radix(v.as_str(), 16)
                    .map_err(|_| error::ColorError::InvalidHex(v.as_str().to_owned()))
            })
    };
}

#[inline]
fn hex_to_rgba(hex: &str) -> Result<Color, error::ColorError> {
    lazy_static::lazy_static! {
        pub static ref HEX_REGEX: Regex = Regex::new(r"^[\s#]*(?P<r>[a-f\d]{2})(?P<g>[a-f\d]{2})(?P<b>[a-f\d]{2})\s*$").unwrap();
    };
    let components = HEX_REGEX
        .captures(hex)
        .ok_or(error::ColorError::InvalidHex(hex.to_string()))?;
    let r = from_hex!(components.name("r"))?;
    let g = from_hex!(components.name("g"))?;
    let b = from_hex!(components.name("b"))?;
    Ok(Color::rgba(r, g, b, 255))
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Copy, Clone)]
pub struct Color {
    rgba: [u8; 4],
}

#[wasm_bindgen]
impl Color {
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(constructor)]
    pub fn hex(hex: &str) -> Result<Color, JsValue> {
        hex_to_rgba(hex).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { rgba: [r, g, b, a] }
    }

    pub fn white() -> Self {
        Self::rgba(255, 255, 255, 255)
    }

    pub fn gray() -> Self {
        Self::rgba(200, 200, 200, 255)
    }
}

impl Color {
    pub fn to_rgba(&self) -> image::Rgba<u8> {
        image::Rgba(self.rgba)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Color {
    pub fn hex(hex: &str) -> Result<Color, error::ColorError> {
        hex_to_rgba(hex)
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[wasm_bindgen]
impl Size {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Size::default()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Point::default()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct Crop {
    pub top: Option<u32>,
    pub right: Option<u32>,
    pub bottom: Option<u32>,
    pub left: Option<u32>,
}

#[wasm_bindgen]
impl Crop {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Crop::default()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct Sides {
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub bottom: u32,
}

#[wasm_bindgen]
impl Sides {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Sides::default()
    }
    pub fn uniform(side: u32) -> Sides {
        Sides {
            top: side,
            left: side,
            right: side,
            bottom: side,
        }
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl std::str::FromStr for Rotation {
    type Err = error::ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "270" => Ok(Rotation::Rotate270),
            "180" => Ok(Rotation::Rotate180),
            "90" => Ok(Rotation::Rotate90),
            "0" => Ok(Rotation::Rotate0),
            _ => Err(error::ParseEnumError::Unknown(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    macro_rules! color_hex_tests {
        ($($name:ident: $values:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (hex, rgba) = $values;
                    assert_eq!(Color::hex(hex).ok(), rgba);
                }
            )*
        }
    }

    color_hex_tests! {
        test_parse_valid_hex_color_1: (
            "#4287f5", Some(Color::rgba(66, 135, 245, 255))),
        test_parse_valid_hex_color_2: (
            "4287f5", Some(Color::rgba(66, 135, 245, 255))),
        test_parse_valid_hex_color_3: (
            "  # 4287f5  ", Some(Color::rgba(66, 135, 245, 255))),
        test_parse_valid_hex_color_4: (
            "#e942f5", Some(Color::rgba(233, 66, 245, 255))),
        test_parse_valid_hex_color_5: (
            "  e942f5", Some(Color::rgba(233, 66, 245, 255))),
        test_parse_invalid_hex_color_1: ("  # 487f5  ", None),
        test_parse_invalid_hex_color_2: ("487f5", None),
        test_parse_invalid_hex_color_3: ("#e942g5", None),
    }
}
