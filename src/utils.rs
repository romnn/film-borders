use super::types;
use num::traits::Float;

pub fn opt_min<T>(v: Option<T>, min: Option<T>) -> Option<T>
where
    T: std::cmp::Ord,
{
    v.map(|v| match min {
        Some(min) => std::cmp::min(v, min),
        None => v,
    })
}

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub trait RoundingMode {
    fn round<F: Float>(value: F) -> F;
}

pub struct Ceil {}
pub struct Floor {}
pub struct Round {}

impl RoundingMode for Round {
    fn round<F: Float>(value: F) -> F {
        value.round()
    }
}

impl RoundingMode for Ceil {
    fn round<F: Float>(value: F) -> F {
        value.ceil()
    }
}

impl RoundingMode for Floor {
    fn round<F: Float>(value: F) -> F {
        value.floor()
    }
}

// pub fn resize_dimensions(
//     size: types::Size,
//     container_size: types::Size,
//     mode: ResizeMode,
// ) -> types::Size {
//     let wratio = container_size.width as f64 / size.width as f64;
//     let hratio = container_size.height as f64 / size.height as f64;
//     let ratio = match mode {
//         ResizeMode::Fill => {
//             return container_size;
//         }
//         ResizeMode::Cover => f64::max(wratio, hratio),
//         ResizeMode::Contain => f64::min(wratio, hratio),
//     };
//     size * ratio

//     // let nw = max((width as f64 * ratio).round() as u64, 1);
//     // let nh = max((height as f64 * ratio).round() as u64, 1);

//     // if nw > u64::from(u32::MAX) {
//     //     let ratio = u32::MAX as f64 / width as f64;
//     //     let nh = max((height as f64 * ratio).round() as u32, 1);
//     //     types::Size {
//     //         width: u32::MAX,
//     //         height: nh,
//     //     }
//     // } else if nh > u64::from(u32::MAX) {
//     //     let ratio = u32::MAX as f64 / height as f64;
//     //     let nw = max((width as f64 * ratio).round() as u32, 1);
//     //     types::Size {
//     //         width: nw,
//     //         height: u32::MAX,
//     //     }
//     // } else {
//     //     types::Size {
//     //         width: nw as u32,
//     //         height: nh as u32,
//     //     }
//     // }
// }

pub fn clamp<T: PartialOrd>(v: T, lower: T, upper: T) -> T {
    if v < lower {
        lower
    } else if v > upper {
        upper
    } else {
        v
    }
}
