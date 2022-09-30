use super::Size;
use crate::numeric::OptionOrd;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Copy, Clone)]
pub struct BoundedSize {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl From<Size> for BoundedSize {
    #[inline]
    fn from(size: Size) -> Self {
        Self {
            width: Some(size.width),
            height: Some(size.height),
        }
    }
}

impl BoundedSize {
    #[inline]
    #[must_use]
    pub fn clamp_min(self, other: Self) -> Self {
        let width = OptionOrd::min(self.width, other.width);
        let height = OptionOrd::min(self.height, other.height);
        Self { width, height }
    }
}

#[wasm_bindgen]
impl BoundedSize {
    #[wasm_bindgen(constructor)]
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        BoundedSize::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bounded_size() {
        assert_eq!(
            &BoundedSize {
                width: Some(10),
                height: None
            }
            .clamp_min(BoundedSize {
                width: Some(12),
                height: Some(10)
            }),
            &BoundedSize {
                width: Some(10),
                height: None
            }
        );
        assert_eq!(
            &BoundedSize {
                width: Some(10),
                height: None
            }
            .clamp_min(BoundedSize {
                width: Some(5),
                height: Some(10)
            }),
            &BoundedSize {
                width: Some(5),
                height: None
            }
        );
    }
}
