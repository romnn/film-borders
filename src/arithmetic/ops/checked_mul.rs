use crate::arithmetic::{
    self,
    error::{Overflow, Underflow},
};
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
impl_unsigned_checked_mul!(u64);

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

impl_float_checked_mul!(f32);
impl_float_checked_mul!(f64);

#[derive(PartialEq, Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct MulError<Lhs, Rhs>(pub arithmetic::error::Operation<Lhs, Rhs>);

impl<Lhs, Rhs> arithmetic::error::Arithmetic for MulError<Lhs, Rhs>
where
    Lhs: arithmetic::Type,
    Rhs: arithmetic::Type,
{
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
            .map(arithmetic::error::AsErr::as_err)
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
