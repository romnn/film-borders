use super::arithmetic::{ops::CheckedAdd, Cast};
use super::img;
use super::types::{Point, Rect, Size};
pub use image::imageops::*;
use image::{Pixel, Rgba};

#[inline]
#[must_use]
pub fn find_transparent_components(
    image: &img::Image,
    alpha_threshold: f32,
    component_threshold: i64,
) -> Vec<Rect> {
    let mut components: Vec<Rect> = Vec::new();
    let alpha_threshold: u8 = (alpha_threshold * 255.0)
        .clamp(0.0, 255.0)
        .cast::<u8>()
        .unwrap();

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
                let contained = c.contains(&point, component_threshold).unwrap();
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
                        if updated.has_intersection(other, 0).unwrap_or(false) {
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
                    components.push(Rect {
                        top: i64::from(y),
                        left: i64::from(x),
                        bottom: i64::from(y),
                        right: i64::from(x),
                    });
                }
            }
        }
    }
    components
}

#[derive(Clone, Copy, Debug)]
pub enum FillMode {
    Blend,
    Set,
}

#[inline]
pub fn fill_rect<TL, S, C>(image: &mut img::Image, color: C, top_left: TL, size: S, mode: FillMode)
where
    TL: Into<Point>,
    S: Into<Size>,
    C: Into<image::Rgba<u8>>,
{
    let top_left = top_left.into();
    let color = color.into();
    let top_left: Size = top_left.try_into().unwrap();

    let bottom_right = top_left.checked_add(size.into()).unwrap();
    let bottom_right = bottom_right.clamp((0, 0), image.size());

    for x in top_left.width..bottom_right.width {
        for y in top_left.height..bottom_right.height {
            let p = image.inner.get_pixel_mut(x, y);
            match mode {
                FillMode::Blend => p.blend(&color),
                FillMode::Set => *p = color,
            }
        }
    }
}

#[inline]
pub fn fade_out<P1, P2>(image: &mut img::Image, start: P1, end: P2, axis: super::Axis)
where
    P1: Into<Point>,
    P2: Into<Point>,
{
    use super::Axis;

    let start = start.into();
    let end = end.into();

    let switch_direction = match axis {
        Axis::X => start.x < end.x,
        Axis::Y => start.y < end.y,
    };
    let switch_direction = if switch_direction { 1.0 } else { 0.0 };

    let rect = Rect::from_points(start, end);
    let size = rect.size().unwrap();
    let top_left = Size::try_from(rect.top_left()).unwrap();
    let dx = top_left.width;
    let dy = top_left.height;

    let (w, h) = match axis {
        Axis::X => (size.height, size.width),
        Axis::Y => (size.width, size.height),
    };
    for y in 0..h {
        let mut frac = f64::from(y) / f64::from(h);
        frac = (switch_direction - frac).abs();
        let alpha = (255.0 * frac).cast::<u8>().unwrap();

        for x in 0..w {
            let (x, y) = match axis {
                Axis::X => (y, x),
                Axis::Y => (x, y),
            };

            let channels = image.inner.get_pixel_mut(dx + x, dy + y).channels_mut();
            channels[3] = channels[3].min(alpha);
        }
    }
}
