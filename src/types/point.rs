use super::Size;
use crate::arithmetic::{
    self,
    ops::{self, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
    Cast, Clamp, ClampMin, Round,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("failed to scale {point} by {scalar:?}")]
pub struct ScaleByError {
    point: Point,
    scalar: Option<f64>,
    source: arithmetic::Error,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl arithmetic::Type for Point {}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Point::default()
    }

    #[inline]
    #[must_use]
    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Point {
    #[inline]
    pub fn scale_by<F, R>(self, scalar: F) -> Result<Self, ScaleByError>
    where
        R: arithmetic::RoundingMode,
        F: arithmetic::Cast + arithmetic::Type,
    {
        match (|| {
            let scalar = scalar.cast::<f64>()?;
            let x = self.x.cast::<f64>()?;
            let y = self.y.cast::<f64>()?;
            let x = CheckedMul::checked_mul(x, scalar)?;
            let y = CheckedMul::checked_mul(y, scalar)?;
            let x = R::round(x).cast::<i64>()?;
            let y = R::round(y).cast::<i64>()?;
            Ok::<Self, arithmetic::Error>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(err) => Err(ScaleByError {
                point: self,
                scalar: scalar.cast::<f64>().ok(),
                source: err,
            }),
        }
    }
}

impl Default for Point {
    #[inline]
    fn default() -> Self {
        Self::origin()
    }
}

impl From<(i64, i64)> for Point {
    #[inline]
    #[must_use]
    fn from(coords: (i64, i64)) -> Self {
        Self {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl From<Size> for Point {
    #[inline]
    #[must_use]
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

impl ClampMin for Point {
    fn clamp_min<MIN>(self, min: MIN) -> Self
    where
        MIN: Into<Self>,
    {
        let min = min.into();
        Self {
            x: num::traits::clamp_min(self.x, min.x),
            y: num::traits::clamp_min(self.y, min.y),
        }
    }
}

impl Clamp for Point {
    fn clamp<MIN, MAX>(self, min: MIN, max: MAX) -> Self
    where
        MIN: Into<Self>,
        MAX: Into<Self>,
    {
        let min = min.into();
        let max = max.into();
        Self {
            x: num::clamp(self.x, min.x, max.x),
            y: num::clamp(self.y, min.y, max.y),
        }
    }
}

impl CheckedAdd for Point {
    type Output = Self;
    type Error = ops::AddError<Self, Self>;

    #[inline]
    fn checked_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        use arithmetic::error::Operation;
        match (|| {
            let x = CheckedAdd::checked_add(self.x, rhs.x)?;
            let y = CheckedAdd::checked_add(self.y, rhs.y)?;
            Ok::<Self, ops::AddError<i64, i64>>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(err) => Err(ops::AddError(Operation {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err.into()),
            })),
        }
    }
}

impl CheckedSub for Point {
    type Output = Self;
    type Error = ops::SubError<Self, Self>;

    #[inline]
    fn checked_sub(self, rhs: Self) -> Result<Self, Self::Error> {
        use arithmetic::error::Operation;
        match (|| {
            let x = CheckedSub::checked_sub(self.x, rhs.x)?;
            let y = CheckedSub::checked_sub(self.y, rhs.y)?;
            Ok::<Self, ops::SubError<i64, i64>>(Self { x, y })
        })() {
            Ok(point) => Ok(point),
            Err(err) => Err(ops::SubError(Operation {
                lhs: self,
                rhs,
                kind: None,
                cause: Some(err.into()),
            })),
        }
    }
}

impl<F> CheckedMul<F> for Point
where
    F: arithmetic::Cast + arithmetic::Type,
{
    type Output = Self;
    type Error = ops::MulError<Self, F>;

    #[inline]
    fn checked_mul(self, scalar: F) -> Result<Self::Output, Self::Error> {
        use arithmetic::error::Operation;
        match self.scale_by::<_, Round>(scalar) {
            Ok(point) => Ok(point),
            Err(ScaleByError { source, .. }) => Err(ops::MulError(Operation {
                lhs: self,
                rhs: scalar,
                kind: None,
                cause: Some(source),
            })),
        }
    }
}

impl<F> CheckedDiv<F> for Point
where
    F: arithmetic::Cast + arithmetic::Type + num::traits::Inv<Output = F>,
{
    type Output = Self;
    type Error = ops::DivError<Self, F>;

    #[inline]
    fn checked_div(self, scalar: F) -> Result<Self::Output, Self::Error> {
        use arithmetic::error::Operation;
        let inverse = scalar.inv();
        match self.scale_by::<_, Round>(inverse) {
            Ok(point) => Ok(point),
            Err(ScaleByError { source, .. }) => Err(ops::DivError(Operation {
                lhs: self,
                rhs: inverse,
                kind: None,
                cause: Some(source),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Point;
    use crate::arithmetic::{ops::CheckedAdd, Ceil, Floor, Round};
    use crate::error::Report;
    use crate::test::assert_matches_regex;
    use pretty_assertions::assert_eq;

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
