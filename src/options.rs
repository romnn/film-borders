use super::types;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Options {
    pub output_size: super::BoundedSize,
    pub output_size_bounds: super::BoundedSize,
    pub scale_factor: f32,
    pub margin: f32,
    pub mode: super::FitMode,
    pub crop: Option<super::sides::percent::Sides>,
    pub frame_width: super::sides::percent::Sides,
    pub image_rotation: super::Rotation,
    pub border_rotation: super::Rotation,
    pub frame_color: super::Color,
    pub background_color: Option<super::Color>,
    pub preview: bool,
}

impl Default for Options {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self {
            output_size: super::BoundedSize::default(),
            output_size_bounds: super::BoundedSize::default(),
            margin: 0.0,
            scale_factor: 1.0,
            mode: super::FitMode::default(),
            crop: None,
            frame_width: super::sides::percent::Sides::default(),
            image_rotation: super::Rotation::default(),
            border_rotation: super::Rotation::default(),
            frame_color: super::Color::black(),
            background_color: None,
            preview: false,
        }
    }
}

#[wasm_bindgen]
impl Options {
    #[wasm_bindgen(constructor)]
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Options::default()
    }

    #[inline]
    pub fn deserialize(val: &str) -> Result<Options, JsError> {
        let options = serde_json::from_str(val)?;
        Ok(options)
        // .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[inline]
    pub fn serialize(&self) -> Result<String, JsError> {
        let json = serde_json::to_string(&self)?;
        Ok(json)
        // .map_err(|err| JsValue::from_str(&err.to_string()))
    }
}

impl super::debug::Private for Options {}

// impl super::debug::AsJson for Options {
//     fn into_json(self) -> Result<JsValue, JsError> {
//         let json = serde_json::to_string(&self)?;
//         let json = js_sys::JSON::parse(json)?;
//         Ok(json)
//     }
// }

// impl TryFrom<Options> for JsValue
// // impl<T> TryInto<JsValue> for T
// // where
// //     T: Serialize,
// {
//     type Error = JsError;

//     fn try_from(options: Options) -> Result<JsValue, Self::Error> {
//         let json = serde_json::to_string(&options)?;
//         let json = js_sys::JSON::parse(json)?;
//         Ok(json)
//         // .map_err(|err| JsError::from_str(&err.to_string()))
//     }
// }

// impl TryInto<JsValue> for Options
// // impl<T> TryInto<JsValue> for T
// // where
// //     T: Serialize,
// {
//     type Error = JsError;

//     fn try_into(self) -> Result<JsValue, Self::Error> {
//         let json = serde_json::to_string(&self)?;
//         let json = js_sys::JSON::parse(json)?;
//         Ok(json)
//         // .map_err(|err| JsError::from_str(&err.to_string()))
//     }
// }

impl Options {
    pub fn background_color(&self) -> types::Color {
        self.background_color.unwrap_or(if self.preview {
            types::Color::gray()
        } else {
            types::Color::white()
        })
    }
}
