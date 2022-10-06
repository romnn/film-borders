use regex::Regex;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid hex color: `{0}`")]
    InvalidHex(String),

    #[error("hex color is missing component")]
    MissingComponent,
}

macro_rules! from_hex {
    ($value:expr) => {{
        let color = $value.ok_or(Error::MissingComponent);
        color.and_then(|hex| match u8::from_str_radix(hex.as_str(), 16) {
            Ok(decimal) => Ok(decimal),
            Err(_) => {
                let invalid = hex.as_str().to_owned();
                Err($crate::types::color::Error::InvalidHex(invalid))
            }
        })
    }};
}

#[inline]
fn hex_to_color(hex: &str) -> Result<Color, Error> {
    lazy_static::lazy_static! {
        pub static ref HEX_REGEX: Regex = Regex::new(r"^[\s#]*(?P<r>[a-f\d]{2})(?P<g>[a-f\d]{2})(?P<b>[a-f\d]{2})\s*$").expect("build hex regex");
    };
    let hex = hex.to_ascii_lowercase();
    let components = HEX_REGEX
        .captures(&hex)
        .ok_or_else(|| Error::InvalidHex(hex.clone()))?;
    let r = from_hex!(components.name("r"))?;
    let g = from_hex!(components.name("g"))?;
    let b = from_hex!(components.name("b"))?;
    Ok(Color::rgba(r, g, b, 255))
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Copy, Clone)]
pub struct Color {
    rgba: [u8; 4],
}

#[wasm_bindgen]
impl Color {
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(constructor)]
    #[inline]
    #[must_use]
    pub fn hex(hex: &str) -> Result<Color, JsValue> {
        hex_to_color(hex).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[inline]
    #[must_use]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            rgba: [r, g, b, 255],
        }
    }

    #[inline]
    #[must_use]
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { rgba: [r, g, b, a] }
    }

    #[inline]
    #[must_use]
    pub fn clear() -> Self {
        Self::rgba(0, 0, 0, 0)
    }

    #[inline]
    #[must_use]
    pub fn black() -> Self {
        Self::rgba(0, 0, 0, 255)
    }

    #[inline]
    #[must_use]
    pub fn white() -> Self {
        Self::rgba(255, 255, 255, 255)
    }

    #[inline]
    #[must_use]
    pub fn gray() -> Self {
        Self::rgba(200, 200, 200, 255)
    }
}

impl From<Color> for image::Rgba<u8> {
    #[inline]
    fn from(color: Color) -> image::Rgba<u8> {
        image::Rgba(color.rgba)
    }
}

impl std::str::FromStr for Color {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex_to_color(s)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Color {
    #[inline]
    pub fn hex(hex: &str) -> Result<Color, Error> {
        hex_to_color(hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
