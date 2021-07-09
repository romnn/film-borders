mod borders;
mod img;
mod utils;

use image::RgbaImage;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log_one(msg: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (console_log_one(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
struct WasmImageBorders {
    // img: img::FilmImage,
    borders: borders::ImageBorders,
    result: Option<RgbaImage>,
}

#[wasm_bindgen]
impl WasmImageBorders {
    #[wasm_bindgen(constructor)]
    pub fn from_canvas(
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<WasmImageBorders, JsValue> {
        utils::set_panic_hook();
        let img = img::FilmImage::from_canvas(&canvas, &ctx)?;
            // .map_err(|err| JsValue::from_str(&err.to_string()))?;
        Ok(WasmImageBorders {
            borders: borders::ImageBorders::new(img),
            result: None,
        })
    }

    pub fn apply(&mut self, options: borders::ImageBorderOptions) -> Result<(), JsValue> {
        self.result = Some(
            self.borders
                .apply(options)
                .map_err(|err| JsValue::from_str(&err.to_string()))?,
        );
        Ok(())
    }

    pub fn update(&self, canvas: HtmlCanvasElement, ctx: CanvasRenderingContext2d) -> Result<(), JsValue> {
        if let Some(result) = &self.result {
            self.borders
                .store(result, canvas, ctx)?
                // .map_err(|err| JsValue::from_str(&err.to_string()))?;
        }
        Ok(())
    }
}
