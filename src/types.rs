use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone)]
pub struct ParseEnumError {
    msg: String,
}

impl ParseEnumError {
    pub fn new(msg: String) -> Self {
        ParseEnumError { msg }
    }
}

impl std::fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for ParseEnumError {}

impl std::str::FromStr for Rotation {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ss = &*s.to_lowercase();
        match ss {
            "270" => Ok(Rotation::Rotate270),
            "180" => Ok(Rotation::Rotate180),
            "90" => Ok(Rotation::Rotate90),
            "0" => Ok(Rotation::Rotate0),
            _ => Err(ParseEnumError::new(format!("unknown rotation: {}", ss))),
        }
    }
}
