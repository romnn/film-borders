use crate::numeric::{error, ArithmeticOp, Numeric};
use num::Zero;
use std::any::Any;
use std::fmt::{self, Debug, Display};

pub trait CheckedDiv<Rhs = Self>
where
    Self: Sized,
{
    type Output;
    type Error;

    fn checked_div(self, scalar: Rhs) -> Result<Self::Output, Self::Error>;
}

macro_rules! impl_unsigned_checked_div {
    ( $T:ty ) => {
        impl CheckedDiv for $T {
            type Output = Self;
            type Error = DivError<Self, Self>;

            fn checked_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
                // can fail if rhs == 0
                if rhs.is_zero() {
                    Err(DivError(self.divide_by_zero()))
                } else {
                    num::CheckedDiv::checked_div(&self, &rhs)
                        .ok_or(rhs.underflows(self))
                        .map_err(DivError)
                }
            }
        }
    };
}

impl_unsigned_checked_div!(u32);

macro_rules! impl_signed_checked_div {
    ( $T:ty ) => {
        impl CheckedDiv for $T {
            type Output = Self;
            type Error = DivError<Self, Self>;

            fn checked_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
                // can fail if rhs == 0
                if rhs.is_zero() {
                    Err(DivError(self.divide_by_zero()))
                } else if self.signum() == rhs.signum() {
                    // can also overflow
                    num::CheckedDiv::checked_div(&self, &rhs)
                        .ok_or(rhs.overflows(self))
                        .map_err(DivError)
                } else {
                    // can also underflow?
                    num::CheckedDiv::checked_div(&self, &rhs)
                        .ok_or(rhs.underflows(self))
                        .map_err(DivError)
                }
            }
        }
    };
}

impl_signed_checked_div!(i64);

macro_rules! impl_float_checked_div {
    ( $T:ty ) => {
        impl CheckedDiv for $T {
            type Output = Self;
            type Error = DivError<Self, Self>;

            fn checked_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
                // can fail if rhs == 0
                if rhs.is_zero() {
                    return Err(DivError(self.divide_by_zero()));
                }
                let result = self / rhs;
                if result.is_nan() && self.signum() == rhs.signum() {
                    // can also overflow
                    Err(DivError(rhs.overflows(self)))
                } else if result.is_nan() {
                    // can also underflow?
                    Err(DivError(rhs.underflows(self)))
                } else {
                    Ok(result)
                }
            }
        }
    };
}

impl_float_checked_div!(f64);

#[derive(PartialEq, Eq, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct DivError<Lhs, Rhs>(pub error::Arithmetic<Lhs, Rhs>);

impl<Lhs, Rhs> error::Numeric for DivError<Lhs, Rhs>
where
    Lhs: Numeric,
    Rhs: Numeric,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &dyn error::Numeric) -> bool {
        match other.as_any().downcast_ref::<Self>() {
            Some(other) => PartialEq::eq(self, other),
            None => false,
        }
    }
}

impl<Lhs, Rhs> std::error::Error for DivError<Lhs, Rhs>
where
    Lhs: Display + Debug,
    Rhs: Display + Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.cause.as_deref().map(error::AsErr::as_err)
    }
}

impl<Lhs, Rhs> Display for DivError<Lhs, Rhs>
where
    Lhs: Display,
    Rhs: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.kind {
            Some(kind) => {
                let kind = match kind {
                    error::Kind::DivideByZero => "is undefined".to_string(),
                    other => other.to_string(),
                };
                write!(
                    f,
                    "dividing {} by {} would {} {}",
                    self.0.lhs,
                    self.0.rhs,
                    kind,
                    std::any::type_name::<Lhs>(),
                )
            }
            None => write!(f, "cannot divide {} by {}", self.0.lhs, self.0.rhs),
        }
    }
}
