use super::arithmetic::{self, ops::CheckedAdd, Cast, CastError, Clamp};
use super::types::{self, Point, Rect, Size};
use super::{error, img};
pub use image::imageops::*;
use image::{GenericImage, GenericImageView, Pixel, Rgba};
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed convert {alpha} to alpha value")]
pub struct AlphaError {
    alpha: f64,
    source: CastError<f64, u8>,
}

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum TransparentComponentsError {
    #[error(transparent)]
    Pad(#[from] types::rect::PadError),

    #[error(transparent)]
    Alpha(#[from] AlphaError),
}

#[inline]
#[must_use]
pub fn find_transparent_components(
    image: &img::Image,
    alpha_threshold: f64,
    component_threshold: u32,
) -> Result<Vec<Rect>, TransparentComponentsError> {
    let mut components: Vec<Rect> = Vec::new();
    let alpha_threshold = (alpha_threshold * 255.0);
    let alpha_threshold = alpha_threshold.cast::<u8>().map_err(|err| AlphaError {
        alpha: alpha_threshold,
        source: err,
    })?;

    let (w, h) = image.inner.dimensions();
    for y in 0..h {
        for x in 0..w {
            let point = Point {
                x: i64::from(x),
                y: i64::from(y),
            };
            let p: &Rgba<u8> = image.inner.get_pixel(x, y);
            let alpha = p.channels()[3];
            if alpha >= alpha_threshold {
                continue;
            }

            let mut updated = None;
            // check if this is a new component
            for c in &mut components {
                let contained = c.padded(component_threshold)?.contains(&point);
                if contained {
                    // update component
                    updated = Some(*c);
                    c.extend_to(&point);
                    break;
                }
            }

            match updated {
                Some(mut updated) => {
                    // merge components
                    // this will remove updated component as well
                    components.retain(|other| {
                        if updated.has_intersection(other) {
                            updated.extend_to(&other.top_left());
                            updated.extend_to(&other.bottom_right());
                            false
                        } else {
                            true
                        }
                    });
                    components.push(updated);
                }
                None => {
                    components.push(Rect::from(point));
                }
            }
        }
    }
    Ok(components)
}

#[derive(Clone, Copy, Debug)]
pub enum FillMode {
    Blend,
    Set,
}

#[inline]
pub fn fill_rect(
    mut image: image::SubImage<&mut image::RgbaImage>,
    color: image::Rgba<u8>,
    mode: FillMode,
) {
    let (w, h) = image.dimensions();
    for x in 0..w {
        for y in 0..h {
            let p = match mode {
                FillMode::Blend => {
                    let mut p = image.get_pixel(x, y);
                    p.blend(&color);
                    p
                }
                FillMode::Set => color,
            };
            image.put_pixel(x, y, p);
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FadeError {
    #[error(transparent)]
    Alpha(#[from] AlphaError),
}

#[inline]
pub fn fade_out(
    mut image: image::SubImage<&mut image::RgbaImage>,
    axis: super::Axis,
    switch_direction: bool,
) -> Result<(), FadeError> {
    use super::Axis;

    let switch_direction = if switch_direction { 1.0 } else { 0.0 };

    let (width, height) = image.dimensions();

    let (w, h) = match axis {
        Axis::X => (height, width),
        Axis::Y => (width, height),
    };
    for y in 0..h {
        let mut frac = f64::from(y) / f64::from(h);
        frac = (switch_direction - frac).abs();
        let alpha = 255.0 * frac;
        let alpha = alpha
            .cast::<u8>()
            .map_err(|err| AlphaError { alpha, source: err })?;

        for x in 0..w {
            let (x, y) = match axis {
                Axis::X => (y, x),
                Axis::Y => (x, y),
            };

            let p = image.get_pixel(x, y);
            let channels = p.channels_mut();
            channels[3] = channels[3].min(alpha);
            image.put_pixel(x, y, p);
        }
    }
    Ok(())
}
