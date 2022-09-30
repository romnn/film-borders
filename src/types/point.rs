use super::*;
use crate::error::*;
use crate::imageops::*;
use crate::numeric::ops::{self, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use crate::numeric::{self, ArithmeticError, ArithmeticOp, NumCast, Round, RoundingMode};
use crate::{img, utils};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::path::Path;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl numeric::NumericType for Point {}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    #[inline]
    pub fn new() -> Self {
        Point::default()
    }

    #[inline]
    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Point {
    #[inline]
    pub fn scale_by<F, R>(self, scalar: F) -> Result<Self, numeric::Error>
    where
        R: RoundingMode,
        F: numeric::NumCast + numeric::NumericType,
    {
        let scalar = scalar.cast::<f64>()?;
        let x = self.x.cast::<f64>()?;
        let y = self.y.cast::<f64>()?;
        let x = CheckedMul::checked_mul(x, scalar)?;
        let y = CheckedMul::checked_mul(y, scalar)?;
        let x = R::round(x).cast::<i64>()?;
        let y = R::round(y).cast::<i64>()?;
        Ok(Self { x, y })
    }
}

impl Default for Point {
    #[inline]
    fn default() -> Self {
        Self::origin()
    }
}

impl From<Size> for Point {
    #[inline]
    fn from(size: Size) -> Self {
        Self {
            x: i64::from(size.width),
            y: i64::from(size.height),
        }
    }
}

impl std::fmt::Display for Point {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

impl CheckedAdd for Point {
    type Output = Self;
    type Error = ops::AddError<Self, Self>;

    #[inline]
    fn checked_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        match (|| {
            let x = CheckedAdd::checked_add(self.x, rhs.x)?;
            let y = CheckedAdd::checked_add(self.y, rhs.y)?;
            Ok::<Self, ops::AddError<i64, i64>>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(err) => Err(ops::AddError(ArithmeticError {
                lhs: self,
                rhs: rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}

// impl std::ops::Add for Point {
//     type Output = Self;

//     fn add(self, rhs: Point) -> Self::Output {
//         Point {
//             x: self.x + rhs.x,
//             y: self.y + rhs.y,
//         }
//     }
// }

impl CheckedSub for Point {
    type Output = Self;
    type Error = ops::SubError<Self, Self>;

    #[inline]
    fn checked_sub(self, rhs: Self) -> Result<Self, Self::Error> {
        match (|| {
            let x = CheckedSub::checked_sub(self.x, rhs.x)?;
            let y = CheckedSub::checked_sub(self.y, rhs.y)?;
            Ok::<Self, ops::SubError<i64, i64>>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(err) => Err(ops::SubError(ArithmeticError {
                lhs: self,
                rhs: rhs,
                kind: None,
                cause: Some(Box::new(err)),
            })),
        }
    }
}

// impl std::ops::Sub for Point {
//     type Output = Self;

//     fn sub(self, rhs: Point) -> Self::Output {
//         Point {
//             x: self.x - rhs.x,
//             y: self.y - rhs.y,
//         }
//     }
// }

impl<F> CheckedMul<F> for Point
where
    F: numeric::NumCast + numeric::NumericType,
{
    type Output = Self;
    type Error = ops::MulError<Self, F>;

    #[inline]
    fn checked_mul(self, scalar: F) -> Result<Point, Self::Error> {
        match (|| {
            let scalar = scalar.cast::<f64>()?;
            let x = self.x.cast::<f64>()?;
            let y = self.y.cast::<f64>()?;
            let x = CheckedMul::checked_mul(x, scalar)?;
            let y = CheckedMul::checked_mul(y, scalar)?;
            let x = x.cast::<i64>()?;
            let y = y.cast::<i64>()?;
            Ok::<Self, numeric::Error>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(numeric::Error(err)) => Err(ops::MulError(ArithmeticError {
                lhs: self,
                rhs: scalar,
                kind: None,
                cause: Some(err),
            })),
        }
    }
}

impl<F> CheckedDiv<F> for Point
where
    F: numeric::NumCast + numeric::NumericType + num::traits::Inv,
{
    type Output = Self;
    type Error = ops::DivError<Self, F>;
    // type Error = numeric::Error;

    #[inline]
    fn checked_div(self, scalar: F) -> Result<Point, Self::Error> {
        match (|| {
            let scalar = scalar.cast::<f64>()?;
            let x = self.x.cast::<f64>()?;
            let y = self.y.cast::<f64>()?;
            let x = CheckedDiv::checked_div(x, scalar)?;
            let y = CheckedDiv::checked_div(y, scalar)?;
            let x = x.cast::<i64>()?;
            let y = y.cast::<i64>()?;
            Ok::<Self, numeric::Error>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(numeric::Error(err)) => Err(ops::DivError(ArithmeticError {
                lhs: self,
                rhs: scalar,
                kind: None,
                cause: Some(err),
            })),
        }
        // use num::traits::Inv;
        // let inverse = scalar.cast::<f64>()?.inv();
        // self.scale_by::<_, Round>(inverse)
    }
}

// impl<F> std::ops::Div<F> for Point
// where
//     F: num::NumCast,
// {
//     type Output = Self;

//     fn div(self, scalar: F) -> Self::Output {
//         let scalar: f64 = num::NumCast::from(scalar).unwrap();
//         self.scale_by::<_, Round>(1.0 / scalar).unwrap()
//     }
// }

#[cfg(test)]
mod tests {
    use super::Point;
    use crate::error::Report;
    use crate::numeric::ops::CheckedAdd;
    use crate::numeric::{Ceil, Floor, Round};
    use crate::test::assert_matches_regex;
    use pretty_assertions::assert_eq;
    use regex::Regex;

    #[test]
    fn scale_by() {
        let p1 = Point { x: 10, y: 10 };
        assert_eq!(p1.scale_by::<_, Round>(0), Ok(Point { x: 0, y: 0 }));
        assert_eq!(p1.scale_by::<_, Round>(2), Ok(Point { x: 20, y: 20 }));
        assert_eq!(p1.scale_by::<_, Round>(-2), Ok(Point { x: -20, y: -20 }));
        assert_eq!(p1.scale_by::<_, Round>(2i128), Ok(Point { x: 20, y: 20 }));
        assert_eq!(p1.scale_by::<_, Round>(1.5), Ok(Point { x: 15, y: 15 }));

        assert_matches_regex!(
            &p1.scale_by::<_, Round>(i128::MIN).err().unwrap().report(),
            r"cannot cast -?\d* of type f64 to i64"
        );
        assert_matches_regex!(
            &p1.scale_by::<_, Round>(u64::MAX).err().unwrap().report(),
            r"cannot cast -?\d* of type f64 to i64"
        );
        assert!(p1.scale_by::<_, Round>(u32::MAX).is_ok());

        let p1 = Point { x: 1, y: 1 };
        assert_eq!(p1.scale_by::<_, Round>(1.5), Ok(Point { x: 2, y: 2 }));
        assert_eq!(p1.scale_by::<_, Ceil>(1.5), Ok(Point { x: 2, y: 2 }));
        assert_eq!(p1.scale_by::<_, Floor>(1.5), Ok(Point { x: 1, y: 1 }));
    }

    #[test]
    fn point_checked_add() {
        let p1 = Point { x: 10, y: 20 };
        let p2 = Point { x: -2, y: 2 };
        assert_eq!(&p1.checked_add(p2).ok().unwrap(), &Point { x: 8, y: 22 });
    }

    #[test]
    fn point_checked_add_underflow() {
        let p1 = Point { x: i64::MIN, y: 0 };
        let p2 = Point { x: -1, y: 0 };
        assert_matches_regex!(
            p1.checked_add(p2).err().unwrap().report(),
            format!(
                r"cannot add {} to {}",
                regex::escape(&p2.to_string()),
                regex::escape(&p1.to_string())
            ),
            r"adding -?\d* to -?\d* would underflow i64"
        );
        let p1 = Point { x: -1, y: i64::MIN };
        let p2 = Point { x: -1, y: -1 };
        assert_matches_regex!(
            &p1.checked_add(p2).err().unwrap().report(),
            format!(
                r"cannot add {} to {}",
                regex::escape(&p2.to_string()),
                regex::escape(&p1.to_string())
            ),
            r"adding -?\d* to -?\d* would underflow i64"
        );
        let p1 = Point { x: -1, y: i64::MIN };
        let p2 = Point { x: -1, y: 0 };
        assert_eq!(
            &p1.checked_add(p2).ok().unwrap(),
            &Point { x: -2, y: i64::MIN },
        );
    }

    #[test]
    fn point_checked_add_overflow() {
        let p1 = Point { x: i64::MAX, y: 0 };
        let p2 = Point { x: 1, y: 0 };
        assert_matches_regex!(
            &p1.checked_add(p2).err().unwrap().report(),
            format!(
                r"cannot add {} to {}",
                regex::escape(&p2.to_string()),
                regex::escape(&p1.to_string())
            ),
            r"adding -?\d* to -?\d* would overflow i64"
        );

        let p1 = Point { x: 1, y: i64::MAX };
        let p2 = Point { x: 1, y: 1 };
        assert_matches_regex!(
            &p1.checked_add(p2).err().unwrap().report(),
            format!(
                r"cannot add {} to {}",
                regex::escape(&p2.to_string()),
                regex::escape(&p1.to_string())
            ),
            r"adding -?\d* to -?\d* would overflow i64"
        );

        let p1 = Point { x: -1, y: i64::MAX };
        let p2 = Point { x: -1, y: 0 };
        assert_eq!(
            &p1.checked_add(p2).ok().unwrap(),
            &Point { x: -2, y: i64::MAX },
        );
    }
}
