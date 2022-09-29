use super::*;
use crate::error::*;
use crate::imageops::*;
use crate::numeric::{self, AddError, ArithmeticError, MulError, SubError};
use crate::numeric::{ArithmeticOp, Round, RoundingMode};
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
    // #[inline]
    // pub fn clamp<P1, P2>(self, min: P1, max: P2) -> Self
    // where
    //     P1: AsRef<Point>,
    //     P2: AsRef<Point>,
    // {
    //     // let min: Point = min.into();
    //     // let max: Point = max.into();
    //     Self {
    //         x: num::clamp(self.x, min.as_ref().x, max.as_ref().x),
    //         // x: utils::clamp(self.x, min.x, max.x),
    //         y: num::clamp(self.y, min.as_ref().y, max.as_ref().y),
    //         // y: utils::clamp(self.y, min.y, max.y),
    //     }
    // }

    #[inline]
    pub fn scale_by<F, R>(self, scalar: F) -> Result<Self, MulError<Self, F>>
    where
        R: RoundingMode,
        F: num::NumCast,
    {
        // match (|| {
        // let scalar: f64 = num::NumCast::from(scalar)?;
        // let x: f64 = num::NumCast::from(self.x)?;
        // let y: f64 = num::NumCast::from(self.y)?;
        // let x = R::round(self.x as f64 * scalar) as u64;
        // let y = R::round(self.y as f64 * scalar) as u64;
        // Self {
        //     x: x as i64,
        //     y: y as i64,
        // }
        Ok(self)
    }

    // #[inline]
    // pub fn magnitude(&self) -> Option<f64> {
    //     // c**2 = a**2 + b**2
    //     let x: f64 = num::NumCast::from(self.x)?;
    //     let y: f64 = num::NumCast::from(self.y)?;
    //     let mag = (x.powi(2) + y.powi(2)).sqrt();
    //     if mag.is_nan() {
    //         None
    //     } else {
    //         Some(mag)
    //     }
    // }

    // #[inline]
    // pub fn abs(self) -> Self {
    //     Self {
    //         x: self.x.abs(),
    //         y: self.y.abs(),
    //     }
    // }
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

impl numeric::CheckedAdd for Point {
    type Output = Self;

    #[inline]
    fn checked_add(&self, rhs: &Point) -> Result<Point, AddError<Self, Self>> {
        match (|| {
            let x = numeric::CheckedAdd::checked_add(&self.x, &rhs.x)?;
            let y = numeric::CheckedAdd::checked_add(&self.y, &rhs.y)?;
            Ok::<Point, _>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(AddError(err)) => Err(AddError(ArithmeticError {
                lhs: *self,
                rhs: *rhs,
                type_name: err.type_name,
                kind: err.kind,
            })),
        }
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

// impl ops::checked::CheckedSub for Point {
//     fn checked_sub(&self, v: &Self) -> Option<Self> {
//         let x = self.x.checked_sub(v.x)?;
//         let y = self.y.checked_sub(v.y)?;
//         Some(Self { x, y })
//     }
// }
impl numeric::CheckedSub for Point {
    type Output = Self;

    #[inline]
    fn checked_sub(&self, rhs: &Point) -> Result<Point, SubError<Self, Self>> {
        match (|| {
            let x = numeric::CheckedSub::checked_sub(&self.x, &rhs.x)?;
            let y = numeric::CheckedSub::checked_sub(&self.y, &rhs.y)?;
            Ok::<Point, _>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(SubError(err)) => Err(SubError(ArithmeticError {
                lhs: *self,
                rhs: *rhs,
                type_name: err.type_name,
                kind: err.kind,
            })),
        }
    }
}

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

// impl numeric::CheckedAdd<Size> for Point {
//     type Output = Point;
//     fn checked_add(&self, size: &Size) -> Result<Point, numeric::AddError<Point, Size>> {
//         // let width: i64 = NumCast::from(size.width)?;
//         // let height: i64 = NumCast::from(size.height)?;
//         let x = self.x.checked_add(width)?;
//         let y = self.y.checked_add(height)?;
//         Some(Self { x, y })
//     }
// }

// impl std::ops::Add<Size> for Point {
//     type Output = Self;

//     fn add(self, size: Size) -> Self::Output {
//         Point {
//             x: self.x + size.width as i64,
//             y: self.y + size.height as i64,
//         }
//     }
// }

impl<F> numeric::CheckedMul<F> for Point
where
    F: num::NumCast,
{
    type Output = Self;

    #[inline]
    fn checked_mul(&self, scalar: &F) -> Result<Point, MulError<Self, F>> {
        Ok(*self)
        // self.scale_by::<_, Round>(scalar)
        // match (|| {
        //     let x = numeric::CheckedSub::checked_sub(&self.x, &rhs.x)?;
        //     let y = numeric::CheckedSub::checked_sub(&self.y, &rhs.y)?;
        //     Ok::<Point, _>(Self { x, y })
        // })() {
        //     Ok(point) => Ok(point),
        //     Err(SubError(err)) => Err(SubError(ArithmeticError {
        //         lhs: *self,
        //         rhs: *rhs,
        //         type_name: err.type_name,
        //         kind: err.kind,
        //     })),
        // }
    }
}

// impl<F> std::ops::Mul<F> for Point
// where
//     F: num::NumCast + std::fmt::Debug,
// {
//     type Output = Self;

//     fn mul(self, scalar: F) -> Self::Output {
//         self.scale_by::<_, Round>(scalar).unwrap()
//     }
// }

impl<F> std::ops::Div<F> for Point
where
    F: num::NumCast,
{
    type Output = Self;

    fn div(self, scalar: F) -> Self::Output {
        let scalar: f64 = num::NumCast::from(scalar).unwrap();
        self.scale_by::<_, Round>(1.0 / scalar).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numeric::CheckedAdd;
    use pretty_assertions::assert_eq;

    #[test]
    fn point_checked_add() {
        let p1 = Point { x: 10, y: 20 };
        let p2 = Point { x: -2, y: 2 };
        assert_eq!(&p1.checked_add(&p2).ok().unwrap(), &Point { x: 8, y: 22 });
    }

    #[test]
    fn point_checked_add_underflow() {
        let p1 = Point { x: i64::MIN, y: 0 };
        let p2 = Point { x: -1, y: 0 };
        assert_eq!(
            &p1.checked_add(&p2).err().unwrap().to_string(),
            &format!("adding {} to {} would underflow i64", &p2, &p1)
        );
        let p1 = Point { x: -1, y: i64::MIN };
        let p2 = Point { x: -1, y: -1 };
        assert_eq!(
            &p1.checked_add(&p2).err().unwrap().to_string(),
            &format!("adding {} to {} would underflow i64", &p2, &p1)
        );
        let p1 = Point { x: -1, y: i64::MIN };
        let p2 = Point { x: -1, y: 0 };
        assert_eq!(
            &p1.checked_add(&p2).ok().unwrap(),
            &Point { x: -2, y: i64::MIN },
        );
    }

    #[test]
    fn point_checked_add_overflow() {
        let p1 = Point { x: i64::MAX, y: 0 };
        let p2 = Point { x: 1, y: 0 };
        assert_eq!(
            &p1.checked_add(&p2).err().unwrap().to_string(),
            &format!("adding {} to {} would overflow i64", &p2, &p1)
        );
        let p1 = Point { x: 1, y: i64::MAX };
        let p2 = Point { x: 1, y: 1 };
        assert_eq!(
            &p1.checked_add(&p2).err().unwrap().to_string(),
            &format!("adding {} to {} would overflow i64", &p2, &p1)
        );
        let p1 = Point { x: -1, y: i64::MAX };
        let p2 = Point { x: -1, y: 0 };
        assert_eq!(
            &p1.checked_add(&p2).ok().unwrap(),
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
