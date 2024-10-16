use crate::{border, builtin, error::Report, img, options};
use image::{DynamicImage, ImageBuffer};
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[inline]
fn image_from_image_data(img: &ImageData) -> Result<DynamicImage, JsError> {
    let pixels = img.data().to_vec();
    let buffer = ImageBuffer::from_vec(img.width(), img.height(), pixels)
        .ok_or_else(|| JsError::new("failed to create image buffer"))?;
    Ok(DynamicImage::ImageRgba8(buffer))
}

#[inline]
fn image_from_canvas(
    canvas: &HtmlCanvasElement,
    ctx: &CanvasRenderingContext2d,
) -> Result<DynamicImage, JsValue> {
    let width = f64::from(canvas.width());
    let height = f64::from(canvas.height());
    let data = ctx.get_image_data(0.0, 0.0, width, height)?;
    let image = image_from_image_data(&data)?;
    Ok(image)
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Border {
    #[cfg(feature = "builtin")]
    builtin: Option<builtin::Builtin>,
    custom: Option<ImageData>,
}

#[wasm_bindgen]
impl Border {
    #[wasm_bindgen(constructor)]
    #[must_use]
    #[inline]
    pub fn new(custom: Option<ImageData>, builtin: Option<builtin::Builtin>) -> Border {
        Border { builtin, custom }
    }

    #[must_use]
    #[inline]
    pub fn from_image_data(data: ImageData) -> Border {
        Border {
            custom: Some(data),
            ..Default::default()
        }
    }
    #[must_use]
    #[inline]
    pub fn builtin(builtin: builtin::Builtin) -> Border {
        Border {
            builtin: Some(builtin),
            ..Default::default()
        }
    }
}

#[wasm_bindgen]
pub struct Image {
    inner: img::Image,
}

#[wasm_bindgen]
impl Image {
    #[inline]
    pub fn from_canvas(
        canvas: &HtmlCanvasElement,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<Image, JsValue> {
        let inner = image_from_canvas(canvas, ctx)?.to_rgba8();
        Ok(Image {
            inner: img::Image { inner, path: None },
        })
    }

    pub fn from_image_data(data: &ImageData) -> Result<Image, JsError> {
        let inner = image_from_image_data(data)?.to_rgba8();
        Ok(Image {
            inner: img::Image { inner, path: None },
        })
    }
}

#[wasm_bindgen]
pub struct ImageBorders {
    inner: crate::ImageBorders,
}

#[wasm_bindgen]
impl ImageBorders {
    #[inline]
    pub fn from_canvas(
        canvas: &HtmlCanvasElement,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<ImageBorders, JsValue> {
        let img = Image::from_canvas(canvas, ctx)?.inner;
        Ok(Self {
            inner: crate::ImageBorders::single(img),
        })
    }

    #[inline]
    pub fn from_image_data(data: &ImageData) -> Result<ImageBorders, JsError> {
        let img = Image::from_image_data(data)?.inner;
        Ok(Self {
            inner: crate::ImageBorders::single(img),
        })
    }

    #[inline]
    pub fn to_image_data(
        canvas: &HtmlCanvasElement,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<ImageData, JsValue> {
        let img = Image::from_canvas(canvas, ctx)?.inner;
        let size = img.size();
        // convert the raw pixels back to an ImageData object
        let data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(img.as_raw()),
            size.width,
            size.height,
        )?;
        Ok(data)
    }

    #[inline]
    pub fn render(
        &mut self,
        border: Border,
        options: &options::Options,
    ) -> Result<ImageData, JsValue> {
        println!("border: {:?}", &border);
        crate::debug!(&options);
        let border = match border.custom {
            None => border.builtin.map(border::Kind::Builtin),
            Some(data) => {
                let image = Image::from_image_data(&data)?;
                let border = border::Border::from_image(image.inner, None)
                    .map(border::Kind::Custom)
                    .map_err(|err| JsError::new(&err.report()))?;
                Some(border)
            }
        };

        let result = self
            .inner
            .render(border, options)
            .map_err(|err| JsError::new(&err.report()))?;
        let size = result.size();
        // convert the raw pixels back to an ImageData object
        let image = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(result.as_raw()),
            size.width,
            size.height,
        )?;
        Ok(image)
    }
}
