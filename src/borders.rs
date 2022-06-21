use lazy_static::lazy_static;
// #![allow(clippy::unused_unit)]
// use crate::img;
// use image::error::ImageError;
// use image::imageops::{crop, overlay, resize, rotate180, rotate270, rotate90, FilterType};
use image::{ImageFormat, RgbaImage};
// use lazy_static::lazy_static;
// use serde::{Deserialize, Serialize};
// use std::cmp::{max, min};
// use std::error;
// use std::fmt;
// use std::path::PathBuf;
// use chrono::Utc;
// use wasm_bindgen::prelude::*;
// use wasm_bindgen::Clamped;
// use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

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
