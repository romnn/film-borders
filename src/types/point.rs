use super::*;
use crate::error::*;
use crate::imageops::*;
use crate::numeric::ops::{AddError, MulError, SubError};
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
        // float multiplication is safe but can result in nans
        let x = x * scalar;
        let y = y * scalar;
        // any nans will be detected when casting back
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

impl numeric::ops::CheckedAdd for Point {
    type Output = Self;

    #[inline]
    fn checked_add(self, rhs: Point) -> Result<Point, AddError<Self, Self>> {
        match (|| {
            let x = numeric::ops::CheckedAdd::checked_add(self.x, rhs.x)?;
            let y = numeric::ops::CheckedAdd::checked_add(self.y, rhs.y)?;
            Ok::<Point, _>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(AddError(err)) => Err(AddError(ArithmeticError {
                lhs: self,
                rhs: rhs,
                type_name: err.type_name,
                kind: err.kind,
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

impl numeric::ops::CheckedSub for Point {
    type Output = Self;

    #[inline]
    fn checked_sub(self, rhs: Point) -> Result<Point, SubError<Self, Self>> {
        match (|| {
            let x = numeric::ops::CheckedSub::checked_sub(self.x, rhs.x)?;
            let y = numeric::ops::CheckedSub::checked_sub(self.y, rhs.y)?;
            Ok::<Point, _>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(SubError(err)) => Err(SubError(ArithmeticError {
                lhs: self,
                rhs: rhs,
                type_name: err.type_name,
                kind: err.kind,
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

impl<F> numeric::ops::CheckedMul<F> for Point
where
    F: numeric::NumCast + numeric::NumericType,
{
    type Output = Self;
    type Error = numeric::Error;

    #[inline]
    fn checked_mul(self, scalar: F) -> Result<Point, Self::Error> {
        self.scale_by::<_, Round>(scalar)
    }
}

impl<F> numeric::ops::CheckedDiv<F> for Point
where
    F: numeric::NumCast + numeric::NumericType + num::traits::Inv,
{
    type Output = Self;
    type Error = numeric::Error;

    #[inline]
    fn checked_div(self, scalar: F) -> Result<Point, Self::Error> {
        use num::traits::Inv;
        let inverse = scalar.cast::<f64>()?.inv();
        self.scale_by::<_, Round>(inverse)
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
    use crate::numeric::ops::CheckedAdd;
    use crate::numeric::{Ceil, Floor, Round};
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
        assert!(Regex::new(r"^cannot cast -?\d* of type f64 to i64$")
            .unwrap()
            .is_match(
                &p1.scale_by::<_, Round>(i128::MIN)
                    .err()
                    .unwrap()
                    .to_string()
            ));
        assert!(Regex::new(r"^cannot cast -?\d* of type f64 to i64$")
            .unwrap()
            .is_match(&p1.scale_by::<_, Round>(u64::MAX).err().unwrap().to_string()));
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
        assert_eq!(
            &p1.checked_add(p2).err().unwrap().to_string(),
            &format!("adding {} to {} would underflow i64", &p2, &p1)
        );
        let p1 = Point { x: -1, y: i64::MIN };
        let p2 = Point { x: -1, y: -1 };
        assert_eq!(
            &p1.checked_add(p2).err().unwrap().to_string(),
            &format!("adding {} to {} would underflow i64", &p2, &p1)
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
        assert_eq!(
            &p1.checked_add(p2).err().unwrap().to_string(),
            &format!("adding {} to {} would overflow i64", &p2, &p1)
        );
        let p1 = Point { x: 1, y: i64::MAX };
        let p2 = Point { x: 1, y: 1 };
        assert_eq!(
            &p1.checked_add(p2).err().unwrap().to_string(),
            &format!("adding {} to {} would overflow i64", &p2, &p1)
        );
        let p1 = Point { x: -1, y: i64::MAX };
        let p2 = Point { x: -1, y: 0 };
        assert_eq!(
            &p1.checked_add(p2).ok().unwrap(),
            &Point { x: -2, y: i64::MAX },
        );
    }

    // macro_rules! color_hex_tests {
    //     ($($name:ident: $values:expr,)*) => {
    //         $(
    //             #[test]
    //             fn $name() {
    //                 let (hex, rgba) = $values;
    //                 assert_eq!(Color::hex(hex).ok(), rgba);
    //             }
    //         )*
    //     }
    // }

    // color_hex_tests! {
    //     test_parse_valid_hex_color_1: (
    //         "#4287f5", Some(Color::rgba(66, 135, 245, 255))),
    //     test_parse_valid_hex_color_2: (
    //         "4287f5", Some(Color::rgba(66, 135, 245, 255))),
    //     test_parse_valid_hex_color_3: (
    //         "  # 4287f5  ", Some(Color::rgba(66, 135, 245, 255))),
    //     test_parse_valid_hex_color_4: (
    //         "#e942f5", Some(Color::rgba(233, 66, 245, 255))),
    //     test_parse_valid_hex_color_5: (
    //         "  e942f5", Some(Color::rgba(233, 66, 245, 255))),
    //     test_parse_invalid_hex_color_1: ("  # 487f5  ", None),
    //     test_parse_invalid_hex_color_2: ("487f5", None),
    //     test_parse_invalid_hex_color_3: ("#e942g5", None),
    // }
}
