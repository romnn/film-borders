use crate::numeric::{error, ArithmeticOp, Numeric};
use std::any::Any;
use std::fmt::{self, Debug, Display};

pub trait CheckedSub<Rhs = Self>
where
    Self: Sized,
{
    type Output;
    type Error;

    fn checked_sub(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

macro_rules! impl_unsigned_checked_sub {
    ( $T:ty ) => {
        impl CheckedSub for $T {
            type Output = Self;
            type Error = SubError<Self, Self>;

            fn checked_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
                num::CheckedSub::checked_sub(&self, &rhs)
                    .ok_or(rhs.underflows(self))
                    .map_err(SubError)
            }
        }
    };
}

impl_unsigned_checked_sub!(u32);

macro_rules! impl_signed_checked_sub {
    ( $T:ty ) => {
        impl CheckedSub for $T {
            type Output = Self;
            type Error = SubError<Self, Self>;

            fn checked_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
                if rhs.is_negative() {
                    num::CheckedAdd::checked_add(&self, &rhs.abs())
                        .ok_or(rhs.overflows(self))
                        .map_err(SubError)
                } else {
                    num::CheckedSub::checked_sub(&self, &rhs)
                        .ok_or(rhs.underflows(self))
                        .map_err(SubError)
                }
            }
        }
    };
}

impl_signed_checked_sub!(i64);

#[derive(PartialEq, Eq, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct SubError<Lhs, Rhs>(pub error::Arithmetic<Lhs, Rhs>);

impl<Lhs, Rhs> error::Numeric for SubError<Lhs, Rhs>
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

impl<Lhs, Rhs> std::error::Error for SubError<Lhs, Rhs>
where
    Lhs: Display + Debug,
    Rhs: Display + Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0
            .cause
            .as_deref()
            .map(error::AsErr::as_err)
    }
}

impl<Lhs, Rhs> Display for SubError<Lhs, Rhs>
where
    Lhs: Display,
    Rhs: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.kind {
            Some(kind) => write!(
                f,
                "subtracting {} from {} would {} {}",
                self.0.rhs,
                self.0.lhs,
                kind,
                std::any::type_name::<Lhs>(),
            ),
            None => write!(f, "cannot subtract {} from {}", self.0.rhs, self.0.lhs),
        }
    }
}
