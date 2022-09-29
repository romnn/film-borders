#[cfg(feature = "borders")]
use crate::borders;
use crate::error;
use crate::imageops::*;
use crate::numeric::{Ceil, Round, RoundingMode};
use crate::{img, utils};
use num::traits::NumCast;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::path::Path;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl Default for Rotation {
    fn default() -> Self {
        Self::Rotate0
    }
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
