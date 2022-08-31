use crate::img::Image;
use crate::options;
use crate::types;
use crate::utils;
use crate::ImageBorders;
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

impl Image {
    #[allow(dead_code)]
    pub fn from_canvas(
        canvas: &HtmlCanvasElement,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<Image, JsValue> {
        let inner = image_from_canvas(canvas, ctx)?.to_rgba8();
        let size = types::Size {
            width: inner.width(),
            height: inner.height(),
        };
        Ok(Image {
            inner,
            path: None,
            size,
        })
    }

    #[allow(dead_code)]
    pub fn from_image_data(data: ImageData) -> Result<Self, JsValue> {
        let inner = image_from_image_data(data)?.to_rgba8();
        let size = types::Size {
            width: inner.width(),
            height: inner.height(),
        };
        Ok(Self {
            inner,
            path: None,
            size,
        })
    }
}

#[allow(dead_code)]
#[wasm_bindgen]
impl ImageBorders {
    // #[allow(dead_code)]
    pub fn from_canvas(
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<ImageBorders, JsValue> {
        Ok(ImageBorders::new(Image::from_canvas(&canvas, &ctx)?))
    }

    // #[allow(dead_code)]
    pub fn for_image_data(data: ImageData) -> Result<ImageBorders, JsValue> {
        Ok(ImageBorders::new(Image::from_image_data(data)?))
    }

    pub fn to_image_data(
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<ImageData, JsValue> {
        utils::set_panic_hook();
        let img = Image::from_canvas(&canvas, &ctx)?;
        let size = img.size();
        // convert the raw pixels back to an ImageData object
        ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(img.as_raw()),
            size.width,
            size.height,
        )
    }

    // #[allow(dead_code)]
    pub fn apply_wasm(&mut self, options: options::BorderOptions) -> Result<ImageData, JsValue> {
        console_log!("options: {:?}", options);
        let result = self
            .apply(options)
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
