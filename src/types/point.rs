use super::*;
#[cfg(feature = "borders")]
use crate::borders;
use crate::error::*;
use crate::imageops::*;
use crate::numeric::{self, ArithmeticOp, Round, RoundingMode};
use crate::{img, utils};
use num::traits::{ops, NumCast};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::path::Path;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl numeric::NumericType for Point {}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

// impl ops::checked::CheckedAdd for Point {
//     fn checked_add(&self, v: &Self) -> Option<Self> {
//         let x = self.x.checked_add(v.x)?;
//         let y = self.y.checked_add(v.y)?;
//         Some(Self { x, y })
//     }
// }

impl numeric::CheckedAdd for Point {
    type Output = Self;

    // fn checked_add(&self, rhs: &Point) -> Result<Point, numeric::Error<Point, Point>> {
    fn checked_add(&self, rhs: &Point) -> Result<Point, numeric::Error> {
        // numeric::WouldOverflowError {
        // let err = numeric::Error::Add(numeric::WouldOverflowError {
        //     left: self,
        //     right: rhs,
        // });
        // let test: Option<Point> = (|| {
        //     let x = self.x.checked_add(rhs.x)?; // .ok_or(err)?;
        //     let y = self.y.checked_add(rhs.y)?; // .ok_or(err)?;
        //     Some(Self { x, y })
        // })();
        match (|| {
            // let x = if rhs.x < 0 {
            //     self.x
            //         .checked_sub(rhs.x.abs())
            //         .ok_or(rhs.underflows::<i64>(self))
            // } else {
            //     self.x.checked_add(rhs.x).ok_or(rhs.overflows::<i64>(self))
            // }?;
            // let y = self.y;
            let x = self.x;
            // let y = CheckedAdd:checked_add(self.y.checked_add(rhs.y)?; // .ok_or(err)?;
            let x = numeric::CheckedAdd::checked_add(&self.x, &rhs.x)?;
            // .ok_or(err)?;
            let y = numeric::CheckedAdd::checked_add(&self.y, &rhs.y)?;
            // .ok_or(err)?;
            Ok::<point::Point, numeric::Error>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(err) => Err(numeric::Error::Add(
                // todo: wrap the error here
                numeric::ArithmeticError {
                    lhs: self,
                    rhs: rhs,
                    ..err
                }, // rhs.overflows::<i64>(self)
            )),
            // err))err), // numeric::Error::Add(err)),
        }
        // Ok(Self { x, y })
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

impl ops::checked::CheckedSub for Point {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        let x = self.x.checked_sub(v.x)?;
        let y = self.y.checked_sub(v.y)?;
        Some(Self { x, y })
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

impl std::ops::Add<Size> for Point {
    type Output = Self;

    fn add(self, size: Size) -> Self::Output {
        Point {
            x: self.x + size.width as i64,
            y: self.y + size.height as i64,
        }
    }
}

impl<F> std::ops::Mul<F> for Point
where
    F: NumCast,
{
    type Output = Self;

    fn mul(self, scalar: F) -> Self::Output {
        self.scale_by::<_, Round>(scalar)
    }
}

impl<F> std::ops::Div<F> for Point
where
    F: NumCast,
{
    type Output = Self;

    fn div(self, scalar: F) -> Self::Output {
        let scalar: f64 = NumCast::from(scalar).unwrap();
        self.scale_by::<_, Round>(1.0 / scalar)
    }
}

impl From<Size> for Point {
    fn from(size: Size) -> Self {
        Self {
            x: size.width as i64,
            y: size.height as i64,
        }
    }
}

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

impl Default for Point {
    #[inline]
    fn default() -> Self {
        Self::origin()
    }
}

impl Point {
    #[inline]
    pub fn clamp<P1, P2>(self, min: P1, max: P2) -> Self
    where
        P1: Into<Point>,
        P2: Into<Point>,
    {
        let min: Point = min.into();
        let max: Point = max.into();
        Self {
            x: utils::clamp(self.x, min.x, max.x),
            y: utils::clamp(self.y, min.y, max.y),
        }
    }

    #[inline]
    pub fn scale_by<F, R>(self, scalar: F) -> Self
    where
        R: RoundingMode,
        F: NumCast,
    {
        let scalar: f64 = NumCast::from(scalar).unwrap();
        let x = R::round(self.x as f64 * scalar) as u64;
        let y = R::round(self.y as f64 * scalar) as u64;
        Self {
            x: x as i64,
            y: y as i64,
        }
    }

    // pub fn unit_vector(self) -> Vector<f64> {
    //     let mag = self.magnitude();
    //     Vector {
    //         x: self.x as f64 / mag,
    //         y: self.y as f64 / mag,
    //     }
    // }

    #[inline]
    pub fn magnitude(&self) -> f64 {
        // c**2 = a**2 + b**2
        let x = (self.x as f64).powi(2);
        let y = (self.y as f64).powi(2);
        (x + y).sqrt()
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
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
        let result = Point { x: 8, y: 22 };
        assert_eq!(p1.checked_add(&p2).ok(), Some(result));
    }

    #[test]
    fn point_checked_add_overflow() {
        let p1 = Point { x: i64::MIN, y: 0 };
        let p2 = Point { x: -1, y: 0 };
        assert_eq!(
            // p1.checked_add(&p2).err().as_ref().map(ToString::to_string),
            p1.checked_add(&p2).err().unwrap().to_string().as_str(),
            ""
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
