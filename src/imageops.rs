use super::arithmetic::{self, ops::CheckedAdd, Cast, Clamp};
use super::types::{Point, Rect, Size};
use super::{error, img};
pub use image::imageops::*;
use image::{Pixel, Rgba};
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[inline]
#[must_use]
pub fn find_transparent_components(
    image: &img::Image,
    alpha_threshold: f32,
    component_threshold: i64,
) -> Result<Vec<Rect>, arithmetic::Error> {
    let mut components: Vec<Rect> = Vec::new();
    // .clamp(0.0, 255.0)
    let alpha_threshold: u8 = (alpha_threshold * 255.0).cast::<u8>()?;

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
                let contained = c.contains_padded(&point, component_threshold)?;
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
    image: &mut img::Image,
    color: image::Rgba<u8>,
    rect: &Rect,
    mode: FillMode,
) -> Result<(), img::Error> {
    let rect = image.subimage_rect(rect)?;
    for x in rect.x_coords() {
        for y in rect.y_coords() {
            let p = image.inner.get_pixel_mut(x, y);
            match mode {
                FillMode::Blend => p.blend(&color),
                FillMode::Set => *p = color,
            }
        }
    }
    Ok(())
}

#[inline]
pub fn fade_out(
    image: &mut img::Image,
    start: impl Into<Point>,
    end: impl Into<Point>,
    axis: super::Axis,
) -> Result<(), img::Error> {
    use super::Axis;

    let start = start.into();
    let end = end.into();

    let switch_direction = match axis {
        Axis::X => start.x < end.x,
        Axis::Y => start.y < end.y,
    };
    let switch_direction = if switch_direction { 1.0 } else { 0.0 };

    let rect = image.subimage_rect(&Rect::from_points(start, end))?;
    let size = rect.size();

    let (w, h) = match axis {
        Axis::X => (size.height, size.width),
        Axis::Y => (size.width, size.height),
    };
    for y in 0..h {
        let mut frac = f64::from(y) / f64::from(h);
        frac = (switch_direction - frac).abs();
        let alpha = 255.0 * frac;
        let alpha = alpha.cast::<u8>().map_err(|err| {
            img::Error::Arithmetic(error::Arithmetic {
                msg: format!("failed to convert {} to alpha value", alpha),
                source: err.into(),
            })
        })?;

        for x in 0..w {
            let (x, y) = match axis {
                Axis::X => (y, x),
                Axis::Y => (x, y),
            };

            let channels = image
                .inner
                .get_pixel_mut(rect.left + x, rect.top + y)
                .channels_mut();
            channels[3] = channels[3].min(alpha);
        }
    }
    Ok(())
}
