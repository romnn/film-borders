use super::border;
use super::error;
use super::{img, Error};
use std::io;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum Border {
    Border120_1,
}

impl Default for Border {
    #[inline]
    fn default() -> Self {
        Border::Border120_1
    }
}

impl std::str::FromStr for Border {
    type Err = error::ParseEnumError;

    fn from_str(s: &str) -> Result<Border, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "120mm" | "120mm1" => Ok(Border::Border120_1),
            _ => Err(error::ParseEnumError::Unknown(s.to_string())),
        }
    }
}

impl Border {
    #[inline]
    pub fn into_border(self) -> Result<border::Border, Error> {
        match self {
            Self::Border120_1 => {
                let data = include_bytes!("../borders/border.png");
                let img = img::Image::from_reader(io::Cursor::new(&data))?;
                border::Border::from_image(img, None)
            }
        }
    }
}

impl Default for border::Kind {
    #[inline]
    fn default() -> Self {
        border::Kind::Builtin(Border::default())
    }
}
