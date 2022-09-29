use super::types::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct Options {
    pub output_size: BoundedSize,
    pub output_size_bounds: BoundedSize,
    pub scale_factor: f32,
    pub margin: f32,
    pub mode: FitMode,
    pub crop: Option<SidesPercent>,
    pub frame_width: SidesPercent,
    pub image_rotation: Rotation,
    pub border_rotation: Rotation,
    pub frame_color: Color,
    pub background_color: Option<Color>,
    pub preview: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            output_size: BoundedSize::default(),
            output_size_bounds: BoundedSize::default(),
            margin: 0.0,
            scale_factor: 1.0,
            mode: FitMode::default(),
            crop: None,
            frame_width: SidesPercent::default(),
            image_rotation: Rotation::default(),
            border_rotation: Rotation::default(),
            frame_color: Color::black(),
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
