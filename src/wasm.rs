use crate::{borders, img, options, types, utils, ImageBorders};
use image::{DynamicImage, ImageBuffer};
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

#[inline]
fn image_from_image_data(img: ImageData) -> Result<DynamicImage, JsValue> {
    let pixels = img.data().to_vec();
    let buffer = ImageBuffer::from_vec(img.width(), img.height(), pixels)
        .ok_or_else(|| JsValue::from_str("failed to create image buffer"))?;
    Ok(DynamicImage::ImageRgba8(buffer))
}

#[inline]
fn image_from_canvas(
    canvas: &HtmlCanvasElement,
    ctx: &CanvasRenderingContext2d,
) -> Result<DynamicImage, JsValue> {
    let width = canvas.width();
    let height = canvas.height();
    let img = ctx.get_image_data(0.0, 0.0, width as f64, height as f64)?;
    image_from_image_data(img)
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Border {
    #[cfg(feature = "borders")]
    builtin: Option<borders::BuiltinBorder>,
    custom: Option<ImageData>,
}

#[wasm_bindgen]
impl Border {
    #[wasm_bindgen(constructor)]
    pub fn new(custom: Option<ImageData>, builtin: Option<borders::BuiltinBorder>) -> Border {
        Border { custom, builtin }
    }

    pub fn from_image_data(data: ImageData) -> Border {
        Border {
            custom: Some(data),
            ..Default::default()
        }
    }
    pub fn builtin(builtin: borders::BuiltinBorder) -> Border {
        Border {
            builtin: Some(builtin),
            ..Default::default()
        }
    }
}

#[wasm_bindgen]
pub struct WasmImage {
    inner: img::Image,
}

#[wasm_bindgen]
impl WasmImage {
    pub fn from_canvas(
        canvas: &HtmlCanvasElement,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<WasmImage, JsValue> {
        utils::set_panic_hook();
        let inner = image_from_canvas(canvas, ctx)?.to_rgba8();
        Ok(WasmImage {
            inner: img::Image { inner, path: None },
        })
    }

    pub fn from_image_data(data: ImageData) -> Result<WasmImage, JsValue> {
        utils::set_panic_hook();
        let inner = image_from_image_data(data)?.to_rgba8();
        Ok(WasmImage {
            inner: img::Image { inner, path: None },
        })
    }
}

#[wasm_bindgen]
pub struct WasmImageBorders {
    inner: ImageBorders,
}

#[wasm_bindgen]
impl WasmImageBorders {
    pub fn from_canvas(
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<WasmImageBorders, JsValue> {
        utils::set_panic_hook();
        let img = WasmImage::from_canvas(&canvas, &ctx)?.inner;
        Ok(WasmImageBorders {
            inner: ImageBorders::single(img),
        })
    }

    pub fn from_image_data(data: ImageData) -> Result<WasmImageBorders, JsValue> {
        utils::set_panic_hook();
        let img = WasmImage::from_image_data(data)?.inner;
        Ok(WasmImageBorders {
            inner: ImageBorders::single(img),
        })
    }

    pub fn to_image_data(
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<ImageData, JsValue> {
        utils::set_panic_hook();
        let img = WasmImage::from_canvas(&canvas, &ctx)?.inner;
        let size = img.size();
        // convert the raw pixels back to an ImageData object
        ImageData::new_with_u8_clamped_array_and_sh(Clamped(img.as_raw()), size.width, size.height)
    }

    pub fn add_border(
        &mut self,
        border: Border,
        options: &options::Options,
    ) -> Result<ImageData, JsValue> {
        console_log!("border: {:?}", &border);
        console_log!("options: {:?}", &options);
        // .unwrap_or(types::BorderSource::default()),
        let border = match border.custom {
            None => border.builtin.map(types::BorderSource::Builtin),
            Some(data) => {
                let image = WasmImage::from_image_data(data)?;
                let border = types::Border::from_image(image.inner, None)
                    .map(types::BorderSource::Custom)
                    .map_err(|err| JsValue::from_str(&err.to_string()))?;
                Some(border)
            }
        };
        console_log!("selected border: {:?}", &border);

        let result = self
            .inner
            .add_border(border, &options)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        let size = result.size();
        // convert the raw pixels back to an ImageData object
        ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(result.as_raw()),
            size.width,
            size.height,
        )
    }
}
