use crate::types;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct Options {
    pub output_size: types::OutputSize,
    pub output_size_bounds: types::OutputSize,
    pub scale_factor: f32,
    pub margin: f32,
    pub mode: types::Mode,
    pub crop: Option<types::SidesPercent>,
    pub frame_width: types::SidesPercent,
    pub rotate_angle: Option<types::Rotation>,
    pub frame_color: types::Color,
    pub background_color: Option<types::Color>,
    // pub output_size: Option<types::OutputSize>,
    // pub output_size_bounds: Option<types::OutputSize>,
    // pub scale_factor: Option<f32>,
    // pub mode: Option<types::Mode>,
    // pub crop: Option<types::SidesPercent>,
    // pub frame_width: Option<types::SidesPercent>,
    // pub rotate_angle: Option<types::Rotation>,
    // pub border_color: Option<types::Color>,
    // pub background_color: Option<types::Color>,
    pub preview: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            output_size: types::OutputSize::default(),
            output_size_bounds: types::OutputSize::default(),
            margin: 0.0,
            scale_factor: 1.0,
            mode: types::Mode::default(),
            crop: None,
            frame_width: types::SidesPercent::default(),
            rotate_angle: None,
            frame_color: types::Color::black(),
            background_color: None,
            preview: false,
        }
    }
}

#[wasm_bindgen]
impl Options {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Options::default()
    }

    #[allow(dead_code)]
    pub fn deserialize(val: String) -> Result<Options, JsValue> {
        serde_json::from_str(&val).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[allow(dead_code)]
    pub fn serialize(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self).map_err(|err| JsValue::from_str(&err.to_string()))
    }
}
