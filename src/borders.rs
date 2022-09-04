use super::error;
use super::types;
use super::{img, Error};
use std::io;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum BuiltinBorder {
    Border120_1,
}

impl Default for BuiltinBorder {
    #[inline]
    fn default() -> Self {
        BuiltinBorder::Border120_1
    }
}

impl std::str::FromStr for BuiltinBorder {
    type Err = error::ParseEnumError;

    fn from_str(s: &str) -> Result<BuiltinBorder, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "120mm" => Ok(BuiltinBorder::Border120_1),
            "120mm1" => Ok(BuiltinBorder::Border120_1),
            _ => Err(error::ParseEnumError::Unknown(s.to_string())),
        }
    }
}

impl BuiltinBorder {
    #[inline]
    pub fn into_border(self) -> Result<types::Border, Error> {
        match self {
            Self::Border120_1 => {
                let data = include_bytes!("border.png");
                let img = img::Image::from_reader(io::Cursor::new(&data))?;
                types::Border::from_image(img, None)
            }
        }
    }
}

impl Default for types::BorderSource {
    #[inline]
    fn default() -> Self {
        types::BorderSource::Builtin(BuiltinBorder::default())
    }
}
