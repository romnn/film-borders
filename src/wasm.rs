use crate::ImageBorders;
use image::{ImageBuffer, DynamicImage};
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use crate::img::Image;
use crate::utils;
use crate::options;
use crate::types;

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
    let img_buffer = ImageBuffer::from_vec(img.width(), img.height(), pixels)
        .ok_or_else(|| JsValue::from_str("failed to create ImageBuffer"))?;
    Ok(DynamicImage::ImageRgba8(img_buffer))
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
        let buffer = image_from_canvas(canvas, ctx)?.to_rgba8();
        let width = buffer.width();
        let height = buffer.height();
        Ok(Image {
            buffer,
            file_path: None,
            size: types::Size { width, height },
        })
    }

    #[allow(dead_code)]
    pub fn from_image_data(data: ImageData) -> Result<Self, JsValue> {
        let buffer = image_from_image_data(data)?.to_rgba8();
        let width = buffer.width();
        let height = buffer.height();
        Ok(Self {
            buffer,
            file_path: None,
            size: types::Size { width, height },
        })
    }
}

#[wasm_bindgen]
impl ImageBorders {
    #[allow(dead_code)]
    pub fn from_canvas(
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<ImageBorders, JsValue> {
        Ok(ImageBorders::new(Image::from_canvas(&canvas, &ctx)?))
    }

    #[allow(dead_code)]
    pub fn for_image_data(data: ImageData) -> Result<ImageBorders, JsValue> {
        Ok(ImageBorders::new(Image::from_image_data(data)?))
    }

    #[allow(dead_code)]
    pub fn to_image_data(
        canvas: HtmlCanvasElement,
        ctx: CanvasRenderingContext2d,
    ) -> Result<ImageData, JsValue> {
        utils::set_panic_hook();
        let img = Image::from_canvas(&canvas, &ctx)?;
        // convert the raw pixels back to an ImageData object
        ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(img.buffer.as_raw()),
            img.buffer.width(),
            img.buffer.height(),
        )
    }

    #[allow(dead_code)]
    pub fn apply_wasm(&mut self, options: options::BorderOptions) -> Result<ImageData, JsValue> {
        console_log!("options: {:?}", options);
        let result = self
            .apply(options)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        self.result = Some(result.clone());
        // convert the raw pixels back to an ImageData object
        ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(result.as_raw()),
            result.width(),
            result.height(),
        )
    }
}
