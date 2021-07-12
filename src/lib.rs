mod borders;
mod img;
mod utils;

use image::RgbaImage;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
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
        Ok(WasmImageBorders {
            borders: borders::ImageBorders::new(img),
            result: None,
        })
    }

    pub fn for_image_data(data: ImageData) -> Result<WasmImageBorders, JsValue> {
        utils::set_panic_hook();
        let img = img::FilmImage::from_image_data(data)?;
        // let img = img::image_from_image_data(data)?;
        Ok(WasmImageBorders {
            borders: borders::ImageBorders::new(img),
            result: None,
        })
    }

    pub fn to_image_data(
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<ImageData, JsValue> {
        utils::set_panic_hook();
        let img = img::FilmImage::from_canvas(&canvas, &ctx)?;
        // convert the raw pixels back to an ImageData object
        Ok(ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(img.buffer.as_raw()),
            img.buffer.width(),
            img.buffer.height(),
        )?)
    }

    pub fn apply(&mut self, options: borders::ImageBorderOptions) -> Result<ImageData, JsValue> {
        console_log!("options: {:?}", options);
        let result = self
            .borders
            .apply(options)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        self.result = Some(result.clone());
        // convert the raw pixels back to an ImageData object
        Ok(ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(result.as_raw()),
            result.width(),
            result.height(),
        )?)
    }

    pub fn update(
        &self,
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<(), JsValue> {
        if let Some(result) = &self.result {
            self.borders.store(result, canvas, ctx)?
        };
        Ok(())
    }
}
