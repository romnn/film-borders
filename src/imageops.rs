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
                            // other.left, other.top);
                            updated.extend_to(other.bottom_right());
                            // other.right, other.bottom);
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
                    // crate::debug!(&components.last());
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
    // crate::debug!(&top_left);

    let bottom_right = top_left + size.into();
    let origin = types::Point::origin();
    let bottom_right = bottom_right.clamp(origin, image.size());
    // crate::debug!(&bottom_right);

    for x in top_left.width..bottom_right.width {
        for y in top_left.height..bottom_right.height {
            image.inner.get_pixel_mut(x, y).blend(&color);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[inline]
pub fn fade_out(image: &mut RgbaImage, start: u32, end: u32, direction: Direction) {
    let other = match direction {
        Direction::Horizontal => image.height(),
        Direction::Vertical => image.width(),
    };
    let diff = (end as f32 - start as f32).abs();
    for i in min(start, end)..=max(start, end) {
        let ir = i - min(start, end);
        let mut frac = ir as f32 / diff;
        if start < end {
            frac = 1.0 - frac;
        }
        let alpha = (255.0 * frac) as u8;
        for j in 0..other {
            let (x, y) = match direction {
                Direction::Horizontal => (i, j),
                Direction::Vertical => (j, i),
            };
            let channels = image.get_pixel_mut(x, y).channels_mut();
            channels[3] = min(channels[3], alpha);
        }
    }
}
