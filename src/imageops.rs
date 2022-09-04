use crate::img;
use crate::types;
use crate::utils;
pub use image::imageops::*;
use image::{Pixel, Rgba, RgbaImage};
use std::cmp::{max, min};

#[inline]
pub fn find_transparent_components(
    image: &img::Image,
    alpha_threshold: f32,
    component_threshold: i64,
) -> Vec<types::Rect> {
    let mut components: Vec<types::Rect> = Vec::new();
    let alpha_threshold: u8 = utils::clamp(alpha_threshold * 255.0, 0.0, 255.0) as u8;
    let (w, h) = image.inner.dimensions();
    for y in 0..h {
        for x in 0..w {
            let p: &Rgba<u8> = image.inner.get_pixel(x, y);
            let alpha = p.channels()[3];
            if alpha >= alpha_threshold {
                continue;
            }

            let mut updated = None;
            // check if this is a new component
            for c in components.iter_mut() {
                if c.contains(x as i64, y as i64, component_threshold) {
                    // update component
                    updated = Some(*c);
                    c.extend_to(types::Point {
                        x: x as i64,
                        y: y as i64,
                    });
                    break;
                }
            }

            match updated {
                Some(mut updated) => {
                    // merge components
                    // this will remove updated component as well
                    components.retain(|other| {
                        if updated.intersects(&other, 0) || other.intersects(&updated, 0) {
                            updated.extend_to(other.top_left());
                            updated.extend_to(other.bottom_right());
                            false
                        } else {
                            true
                        }
                    });
                    components.push(updated);
                }
                None => {
                    components.push(types::Rect {
                        top: y as i64,
                        bottom: y as i64,
                        left: x as i64,
                        right: x as i64,
                    });
                }
            }
        }
    }
    components
}

#[inline]
pub fn fill_rect<TL, S, C>(image: &mut img::Image, color: C, top_left: TL, size: S)
where
    TL: Into<types::Point>,
    S: Into<types::Size>,
    C: Into<image::Rgba<u8>>,
{
    let top_left = top_left.into();
    let color = color.into();
    let top_left: types::Size = top_left.into();

    let bottom_right = top_left + size.into();
    let origin = types::Point::origin();
    let bottom_right = bottom_right.clamp(origin, image.size());

    for x in top_left.width..bottom_right.width {
        for y in top_left.height..bottom_right.height {
            image.inner.get_pixel_mut(x, y).blend(&color);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Axis {
    X,
    Y,
}

#[inline]
pub fn fade_out<P1, P2>(image: &mut img::Image, start: P1, end: P2, axis: Axis)
where
    P1: Into<types::Point>,
    P2: Into<types::Point>,
{
    let start = start.into();
    let end = end.into();

    let switch_direction = match axis {
        Axis::X => start.x < end.x,
        Axis::Y => start.y < end.y,
    };
    let switch_direction = if switch_direction { 1.0 } else { 0.0 };

    let rect = types::Rect::from_points(start, end);
    let size = rect.size();
    let top_left = rect.top_left();
    let dx = max(0, top_left.x) as u32;
    let dy = max(0, top_left.y) as u32;

    let (w, h) = match axis {
        Axis::X => (size.height, size.width),
        Axis::Y => (size.width, size.height),
    };
    for y in 0..h {
        let mut frac = y as f32 / h as f32;
        frac = (switch_direction - frac).abs();
        let alpha = (255.0 * frac) as u8;

        for x in 0..w {
            let (x, y) = match axis {
                Axis::X => (y, x),
                Axis::Y => (x, y),
            };

            let channels = image.inner.get_pixel_mut(dx + x, dy + y).channels_mut();
            channels[3] = min(channels[3], alpha);
        }
    }
}
