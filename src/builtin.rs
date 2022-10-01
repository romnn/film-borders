use super::border;
use super::error;
use super::{img, Error};
use std::io;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum Builtin {
    Border120_1,
}

impl Default for Builtin {
    #[inline]
    fn default() -> Self {
        Self::Border120_1
    }
}

impl std::str::FromStr for Builtin {
    type Err = error::ParseEnum;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "120mm" | "120mm1" => Ok(Self::Border120_1),
            _ => Err(error::ParseEnum::Unknown(s.to_string())),
        }
    }
}

impl Builtin {
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
        border::Kind::Builtin(Builtin::default())
    }
}
