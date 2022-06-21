use crate::types;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct BorderOptions {
    pub output_size: Option<types::OutputSize>,
    pub scale_factor: Option<f32>,
    pub crop: Option<types::Crop>,
    pub border_width: Option<types::Sides>,
    pub rotate_angle: Option<types::Rotation>,
    pub preview: bool,
}

#[wasm_bindgen]
impl BorderOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        BorderOptions::default()
    }

    #[allow(dead_code)]
    pub fn deserialize(val: String) -> Result<BorderOptions, JsValue> {
        serde_json::from_str(&val).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[allow(dead_code)]
    pub fn serialize(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self).map_err(|err| JsValue::from_str(&err.to_string()))
    }
}
