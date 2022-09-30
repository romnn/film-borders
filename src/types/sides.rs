pub mod percent {
    use crate::numeric::ops::{self, CheckedMul};
    use crate::numeric::{self, cast::NumCast, error};
    use crate::types::{Point, Size};
    use serde::{Deserialize, Serialize};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
    pub struct Sides {
        pub top: f32,
        pub left: f32,
        pub right: f32,
        pub bottom: f32,
    }

    #[wasm_bindgen]
    impl Sides {
        #[wasm_bindgen(constructor)]
        #[inline]
        pub fn new() -> Self {
            Self::default()
        }

        #[inline]
        pub fn uniform(side: f32) -> Self {
            Self {
                top: side,
                left: side,
                right: side,
                bottom: side,
            }
        }
    }

    #[inline]
    fn percent_to_abs(percent: f32, dimension: u32) -> Result<u32, numeric::CastError<f64, u32>> {
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
                Ok::<Self::Output, numeric::Error>(Self::Output {
                    top,
                    left,
                    bottom,
                    right,
                })
            })() {
                Ok(sides) => Ok(sides),
                Err(numeric::Error(err)) => Err(ops::MulError(error::ArithmeticError {
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
                Ok::<Self::Output, numeric::Error>(Self::Output {
                    top,
                    left,
                    bottom,
                    right,
                })
            })() {
                Ok(sides) => Ok(sides),
                Err(numeric::Error(err)) => Err(ops::MulError(error::ArithmeticError {
                    lhs: self,
                    rhs: size,
                    kind: None,
                    cause: Some(err),
                })),
            }
        }
    }

    // impl std::ops::Mul<u32> for Sides {
    //     type Output = super::abs::Sides;

    //     fn mul(self, scalar: u32) -> Self::Output {
    //         Self::Output {
    //             top: percent_to_abs(self.top, scalar),
    //             left: percent_to_abs(self.left, scalar),
    //             bottom: percent_to_abs(self.bottom, scalar),
    //             right: percent_to_abs(self.right, scalar),
    //         }
    //     }
    // }

    // impl std::ops::Mul<Size> for Sides {
    //     type Output = super::abs::Sides;

    //     fn mul(self, size: Size) -> Self::Output {
    //         Self::Output {
    //             top: percent_to_abs(self.top, size.height),
    //             left: percent_to_abs(self.left, size.width),
    //             bottom: percent_to_abs(self.bottom, size.height),
    //             right: percent_to_abs(self.right, size.width),
    //         }
    //     }
    // }
}

pub mod abs {
    use crate::numeric::ops::{self, CheckedAdd, CheckedMul};
    use crate::numeric::{self, cast::NumCast, error};
    use crate::types::{Point, Size};

    #[derive(Debug, Default, Copy, Clone)]
    pub struct Sides {
        pub top: u32,
        pub left: u32,
        pub right: u32,
        pub bottom: u32,
    }

    impl Sides {
        #[inline]
        pub fn uniform(side: u32) -> Self {
            Self {
                top: side,
                left: side,
                right: side,
                bottom: side,
            }
        }

        #[inline]
        pub fn height(&self) -> u32 {
            self.top + self.bottom
        }

        #[inline]
        pub fn width(&self) -> u32 {
            self.left + self.right
        }

        #[inline]
        pub fn top_left(&self) -> Point {
            Point {
                x: self.left as i64,
                y: self.top as i64,
            }
        }

        #[inline]
        pub fn bottom_right(&self) -> Point {
            Point {
                x: self.right as i64,
                y: self.bottom as i64,
            }
        }
    }

    // impl CheckedAdd for Sides {
    //     type Output = Self;
    //     type Error = ops::AddError<Self, Self>;

    //     #[inline]
    //     fn checked_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
    //         match (|| {
    //             let top = CheckedAdd::checked_add(self.top, rhs.top)?;
    //             let left = CheckedAdd::checked_add(self.left, rhs.left)?;
    //             let bottom = CheckedAdd::checked_add(self.bottom, rhs.bottom)?;
    //             let right = CheckedAdd::checked_add(self.right, rhs.right)?;
    //             Ok::<Self::Output, ops::AddError<u32, u32>>(Self::Output {
    //                 top,
    //                 left,
    //                 bottom,
    //                 right,
    //             })
    //         })() {
    //             Ok(sides) => Ok(sides),
    //             Err(err) => Err(ops::AddError(error::ArithmeticError {
    //                 lhs: self,
    //                 rhs: rhs,
    //                 kind: None,
    //                 cause: Some(Box::new(err)),
    //             })),
    //         }
    //     }
    // }

    // impl std::ops::Add for Sides {
    //     type Output = Self;

    //     fn add(self, side: Self) -> Self::Output {
    //         Self {
    //             top: self.top + side.top,
    //             right: self.right + side.right,
    //             bottom: self.bottom + side.bottom,
    //             left: self.left + side.left,
    //         }
    //     }
    // }

    impl<F> CheckedMul<F> for Sides
    where
        F: numeric::NumCast + numeric::NumericType,
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

                Ok::<Self::Output, numeric::Error>(Self::Output {
                    top,
                    left,
                    bottom,
                    right,
                })
            })() {
                Ok(sides) => Ok(sides),
                Err(numeric::Error(err)) => Err(ops::MulError(error::ArithmeticError {
                    lhs: self,
                    rhs: scalar,
                    kind: None,
                    cause: Some(err),
                })),
            }
        }
    }

    // impl<F> std::ops::Mul<F> for Sides
    // where
    //     F: num::NumCast,
    // {
    //     type Output = Self;

    //     fn mul(self, scalar: F) -> Self::Output {
    //         let scalar: f32 = num::NumCast::from(scalar).unwrap();
    //         Self {
    //             top: (self.top as f32 * scalar).ceil() as u32,
    //             right: (self.right as f32 * scalar).ceil() as u32,
    //             bottom: (self.bottom as f32 * scalar).ceil() as u32,
    //             left: (self.left as f32 * scalar).ceil() as u32,
    //         }
    //     }
    // }
}
