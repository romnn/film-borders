pub mod percent {
    use crate::arithmetic::{
        self,
        ops::{self, CheckedMul},
        Cast,
    };
    use crate::types::Size;
    use serde::{Deserialize, Serialize};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Serialize, Deserialize, PartialEq, Debug, Default, Copy, Clone)]
    pub struct Sides {
        pub top: f32,
        pub left: f32,
        pub bottom: f32,
        pub right: f32,
    }

    impl arithmetic::Type for Sides {}

    #[wasm_bindgen]
    impl Sides {
        #[wasm_bindgen(constructor)]
        #[inline]
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }

        #[inline]
        #[must_use]
        pub fn uniform(side: f32) -> Self {
            Self {
                top: side,
                left: side,
                right: side,
                bottom: side,
            }
        }
    }

    impl std::fmt::Display for Sides {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_struct("SidesPercent")
                .field("top", &self.top)
                .field("left", &self.left)
                .field("bottom", &self.bottom)
                .field("right", &self.right)
                .finish()
        }
    }

    #[inline]
    fn percent_to_abs(
        percent: f32,
        dimension: u32,
    ) -> Result<u32, arithmetic::CastError<f64, u32>> {
        let percent = f64::from(percent).max(0.0);
        let dimension = f64::from(dimension);
        let absolute = if percent <= 1.0 {
            percent * dimension
        } else {
            percent
        };
        let absolute = absolute.clamp(0.0, dimension).ceil();
        absolute.cast::<u32>()
    }

    impl CheckedMul<u32> for Sides {
        type Output = super::abs::Sides;
        type Error = ops::MulError<Self, u32>;

        #[inline]
        fn checked_mul(self, scalar: u32) -> Result<Self::Output, Self::Error> {
            match (|| {
                let top = percent_to_abs(self.top, scalar)?;
                let left = percent_to_abs(self.left, scalar)?;
                let bottom = percent_to_abs(self.bottom, scalar)?;
                let right = percent_to_abs(self.right, scalar)?;
                Ok::<Self::Output, arithmetic::Error>(Self::Output {
                    top,
                    left,
                    bottom,
                    right,
                })
            })() {
                Ok(sides) => Ok(sides),
                Err(arithmetic::Error(err)) => Err(ops::MulError(arithmetic::error::Arithmetic {
                    lhs: self,
                    rhs: scalar,
                    kind: None,
                    cause: Some(err),
                })),
            }
        }
    }

    impl CheckedMul<Size> for Sides {
        type Output = super::abs::Sides;
        type Error = ops::MulError<Self, Size>;

        #[inline]
        fn checked_mul(self, size: Size) -> Result<Self::Output, Self::Error> {
            match (|| {
                let top = percent_to_abs(self.top, size.height)?;
                let left = percent_to_abs(self.left, size.width)?;
                let bottom = percent_to_abs(self.bottom, size.height)?;
                let right = percent_to_abs(self.right, size.width)?;
                Ok::<Self::Output, arithmetic::Error>(Self::Output {
                    top,
                    left,
                    bottom,
                    right,
                })
            })() {
                Ok(sides) => Ok(sides),
                Err(arithmetic::Error(err)) => Err(ops::MulError(arithmetic::error::Arithmetic {
                    lhs: self,
                    rhs: size,
                    kind: None,
                    cause: Some(err),
                })),
            }
        }
    }
}

pub mod abs {
    use crate::arithmetic::{
        self,
        ops::{self, CheckedMul},
        Cast,
    };
    use crate::types::Point;

    #[derive(PartialEq, Eq, Debug, Default, Copy, Clone)]
    pub struct Sides {
        pub top: u32,
        pub left: u32,
        pub bottom: u32,
        pub right: u32,
    }

    impl arithmetic::Type for Sides {}

    impl Sides {
        #[inline]
        #[must_use]
        pub fn uniform(side: u32) -> Self {
            Self {
                top: side,
                left: side,
                right: side,
                bottom: side,
            }
        }

        #[inline]
        #[must_use]
        pub fn height(&self) -> u32 {
            self.top.checked_add(self.bottom).unwrap()
        }

        #[inline]
        #[must_use]
        pub fn width(&self) -> u32 {
            self.left.checked_add(self.right).unwrap()
        }

        #[inline]
        #[must_use]
        pub fn top_left(&self) -> Point {
            Point {
                x: i64::from(self.left),
                y: i64::from(self.top),
            }
        }

        #[inline]
        #[must_use]
        pub fn bottom_right(&self) -> Point {
            Point {
                x: i64::from(self.right),
                y: i64::from(self.bottom),
            }
        }
    }

    impl std::fmt::Display for Sides {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_struct("Sides")
                .field("top", &self.top)
                .field("left", &self.left)
                .field("bottom", &self.bottom)
                .field("right", &self.right)
                .finish()
        }
    }

    impl<F> CheckedMul<F> for Sides
    where
        F: arithmetic::Cast + arithmetic::Type,
    {
        type Output = Self;
        type Error = ops::MulError<Self, F>;

        #[inline]
        fn checked_mul(self, scalar: F) -> Result<Self::Output, Self::Error> {
            match (|| {
                let scalar = scalar.cast::<f64>()?;
                let top = self.top.cast::<f64>()?;
                let left = self.left.cast::<f64>()?;
                let bottom = self.bottom.cast::<f64>()?;
                let right = self.right.cast::<f64>()?;

                let top = CheckedMul::checked_mul(top, scalar)?;
                let left = CheckedMul::checked_mul(left, scalar)?;
                let bottom = CheckedMul::checked_mul(bottom, scalar)?;
                let right = CheckedMul::checked_mul(right, scalar)?;

                let top = top.ceil().cast::<u32>()?;
                let left = left.ceil().cast::<u32>()?;
                let bottom = bottom.ceil().cast::<u32>()?;
                let right = right.ceil().cast::<u32>()?;

                Ok::<Self::Output, arithmetic::Error>(Self::Output {
                    top,
                    left,
                    bottom,
                    right,
                })
            })() {
                Ok(sides) => Ok(sides),
                Err(arithmetic::Error(err)) => Err(ops::MulError(arithmetic::error::Arithmetic {
                    lhs: self,
                    rhs: scalar,
                    kind: None,
                    cause: Some(err),
                })),
            }
        }
    }
}
