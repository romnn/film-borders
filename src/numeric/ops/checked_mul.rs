use crate::numeric::{error, ArithmeticOp, NumericType};
use std::any::Any;
use std::fmt::{self, Debug, Display};

pub trait CheckedMul<Rhs = Self>
where
    Self: Sized,
{
    type Output;
    type Error;

    fn checked_mul(self, scalar: Rhs) -> Result<Self::Output, Self::Error>;
}

macro_rules! impl_unsigned_checked_mul {
    ( $T:ty ) => {
        impl CheckedMul for $T {
            type Output = Self;
            type Error = MulError<Self, Self>;

            fn checked_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
                num::CheckedMul::checked_mul(&self, &rhs)
                    .ok_or(rhs.overflows(self))
                    .map_err(MulError)
            }
        }
    };
}

impl_unsigned_checked_mul!(u32);

macro_rules! impl_signed_checked_mul {
    ( $T:ty ) => {
        impl CheckedMul for $T {
            type Output = Self;
            type Error = MulError<Self, Self>;

            fn checked_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
                if self.signum() == rhs.signum() {
                    num::CheckedMul::checked_mul(&self, &rhs)
                        .ok_or(rhs.overflows(self))
                        .map_err(MulError)
                } else {
                    num::CheckedMul::checked_mul(&self, &rhs)
                        .ok_or(rhs.underflows(self))
                        .map_err(MulError)
                }
            }
        }
    };
}

impl_signed_checked_mul!(i64);

macro_rules! impl_float_checked_mul {
    ( $T:ty ) => {
        impl CheckedMul for $T {
            type Output = Self;
            type Error = MulError<Self, Self>;

            fn checked_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
                let result = self * rhs;
                if result.is_nan() && self.signum() == rhs.signum() {
                    // overflow
                    Err(MulError(rhs.overflows(self)))
                } else if result.is_nan() {
                    // underflow
                    Err(MulError(rhs.underflows(self)))
                } else {
                    Ok(result)
                }
            }
        }
    };
}

impl_float_checked_mul!(f64);

#[derive(PartialEq, Eq, Debug)]
pub struct MulError<Lhs, Rhs>(pub error::ArithmeticError<Lhs, Rhs>);

impl<Lhs, Rhs> error::NumericError for MulError<Lhs, Rhs>
where
    Lhs: NumericType,
    Rhs: NumericType,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &dyn error::NumericError) -> bool {
        match other.as_any().downcast_ref::<Self>() {
            Some(other) => PartialEq::eq(self, other),
            None => false,
        }
    }
}

impl<Lhs, Rhs> std::error::Error for MulError<Lhs, Rhs>
where
    Lhs: Display + Debug,
    Rhs: Display + Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0
            .cause
            .as_deref()
            .map(error::AsError::as_error)
    }
}

impl<Lhs, Rhs> Display for MulError<Lhs, Rhs>
where
    Lhs: Display,
    Rhs: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.kind {
            Some(kind) => write!(
                f,
                "multiplying {} by {} would {} {}",
                self.0.lhs,
                self.0.rhs,
                kind,
                std::any::type_name::<Lhs>(),
            ),
            None => write!(f, "cannot multiply {} by {}", self.0.lhs, self.0.rhs,),
        }
    }
}
