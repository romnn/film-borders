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

pub fn clamp<T: PartialOrd>(v: T, lower: T, upper: T) -> T {
    if v < lower {
        lower
    } else if v > upper {
        upper
    } else {
        v
    }
}
