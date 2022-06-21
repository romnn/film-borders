use lazy_static::lazy_static;
use image::{ImageFormat, RgbaImage};

lazy_static! {
    pub static ref BORDER1: RgbaImage = {
        let border = include_bytes!("border.png");
        image::load_from_memory_with_format(border, ImageFormat::Png)
            .expect("decode film border")
            .to_rgba8()
    };
}

pub enum BorderOverlay {
    Border1,
    Custom(RgbaImage),
}
