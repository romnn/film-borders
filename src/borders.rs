#[cfg(feature = "borders")]
use super::error;
use super::{img, Error};
#[cfg(feature = "borders")]
use std::io;
use std::path::Path;
#[cfg(feature = "borders")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "borders")]
#[wasm_bindgen]
#[derive(Debug)]
pub enum Builtin {
    Border120_1,
}

#[cfg(feature = "borders")]
impl std::str::FromStr for Builtin {
    type Err = error::ParseEnumError;

    fn from_str(s: &str) -> Result<Builtin, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "120mm" => Ok(Builtin::Border120_1),
            "120mm1" => Ok(Builtin::Border120_1),
            _ => Err(error::ParseEnumError::Unknown(s.to_string())),
        }
    }
}

#[cfg(feature = "borders")]
impl Builtin {
    #[inline]
    pub fn into_image(self) -> Result<img::Image, Error> {
        match self {
            Self::Border120_1 => {
                let data = include_bytes!("border.png");
                img::Image::new(io::Cursor::new(&data))
            }
        }
    }
}

#[derive(Debug)]
pub enum Border {
    #[cfg(feature = "borders")]
    Builtin(Builtin),
    Custom(img::Image),
}

#[cfg(feature = "borders")]
impl Default for Border {
    #[inline]
    fn default() -> Self {
        Self::Builtin(Builtin::Border120_1)
    }
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
