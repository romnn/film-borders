use super::arithmetic::{
    self,
    ops::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
    Cast, CastError, Clamp,
};
use super::types::{self, Point, Rect, Size};
use super::{error, img};
pub use image::imageops::*;
use image::{GenericImage, GenericImageView, Pixel, Rgba};
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum FillMode {
    Blend,
    Set,
}

#[inline]
#[must_use]
pub fn find_transparent_components(
    image: &img::Image,
    alpha_threshold: f64,
    component_threshold: u32,
) -> Result<Vec<Rect>, TransparentComponentsError> {
    let mut components: Vec<Rect> = Vec::new();
    let alpha_threshold =
        CheckedMul::checked_mul(alpha_threshold, 255.0).map_err(arithmetic::Error::from)?;
    let alpha_threshold = alpha_threshold
        .cast::<u8>()
        .map_err(arithmetic::Error::from)?;

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
                let padded = c
                    .padded(component_threshold)
                    .map_err(arithmetic::Error::from)?;
                if padded.contains(&point) {
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

#[inline]
pub fn clip_alpha(mut image: image::SubImage<&mut image::RgbaImage>, min_alpha: u8, max_alpha: u8) {
    let (w, h) = image.dimensions();
    for x in 0..w {
        for y in 0..h {
            let mut p = image.get_pixel(x, y);
            let channels = p.channels_mut();
            channels[3] = Clamp::clamp(channels[3], min_alpha, max_alpha);
            image.put_pixel(x, y, p);
        }
    }
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
        let alpha = (|| {
            let mut frac = CheckedDiv::checked_div(f64::from(y), f64::from(h))?;
            frac = switch_direction - frac;
            frac = frac.abs();
            let alpha = CheckedMul::checked_mul(frac, 255.0)?;
            let alpha = alpha.cast::<u8>()?;
            Ok::<_, arithmetic::Error>(alpha)
        })();
        let alpha = alpha?;

        for x in 0..w {
            let (x, y) = match axis {
                Axis::X => (y, x),
                Axis::Y => (x, y),
            };

            let mut p = image.get_pixel(x, y);
            let channels = p.channels_mut();
            channels[3] = channels[3].min(alpha);
            image.put_pixel(x, y, p);
        }
    }
    Ok(())
}

#[derive(thiserror::Error, PartialEq, Clone, Debug)]
pub enum TransparentComponentsError {
    #[error(transparent)]
    Arithmetic(#[from] arithmetic::Error),
}

impl arithmetic::error::Arithmetic for TransparentComponentsError {}

#[derive(thiserror::Error, PartialEq, Clone, Debug)]
pub enum FadeError {
    #[error(transparent)]
    Arithmetic(#[from] arithmetic::Error),
}

impl arithmetic::error::Arithmetic for FadeError {}
