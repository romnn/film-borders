use crate::types;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Options {
    pub output_size: Option<types::OutputSize>,
    pub scale_factor: Option<f32>,
    pub crop: Option<types::Crop>,
    pub border_width: Option<types::Sides>,
    pub rotate_angle: Option<types::Rotation>,
    pub background_color: Option<types::Color>,
    pub preview: bool,
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
