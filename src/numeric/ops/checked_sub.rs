use crate::numeric::{error, ArithmeticOp, NumericType};
use std::any::Any;
use std::fmt::{self, Debug, Display};

pub trait CheckedSub<Rhs = Self>
where
    Self: Sized,
{
    type Output;

    fn checked_sub(self, rhs: Rhs) -> Result<Self::Output, SubError<Self, Rhs>>;
}

macro_rules! impl_unsigned_checked_sub {
    ( $T:ty ) => {
        impl CheckedSub for $T {
            type Output = $T;
            fn checked_sub(self, rhs: Self) -> Result<Self::Output, SubError<Self, Self>> {
                num::CheckedSub::checked_sub(&self, &rhs)
                    .ok_or(rhs.underflows::<$T>(self))
                    .map_err(SubError)
            }
        }
    };
}

macro_rules! impl_signed_checked_sub {
    ( $T:ty ) => {
        impl CheckedSub for $T {
            type Output = $T;
            fn checked_sub(self, rhs: Self) -> Result<Self::Output, SubError<$T, $T>> {
                if rhs.is_negative() {
                    num::CheckedAdd::checked_add(&self, &rhs.abs())
                        .ok_or(rhs.overflows::<$T>(self))
                        .map_err(SubError)
                } else {
                    num::CheckedSub::checked_sub(&self, &rhs)
                        .ok_or(rhs.underflows::<$T>(self))
                        .map_err(SubError)
                }
            }
        }
    };
}

// impl_unsigned_checked_sub!(u32);
impl_signed_checked_sub!(i64);

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub struct SubError<Lhs, Rhs>(pub error::ArithmeticError<Lhs, Rhs>);

impl<Lhs, Rhs> From<SubError<Lhs, Rhs>> for error::Error
where
    Lhs: NumericType,
    Rhs: NumericType,
{
    fn from(err: SubError<Lhs, Rhs>) -> Self {
        error::Error::Sub(Box::new(err))
    }
}

impl<Lhs, Rhs> error::NumericError for SubError<Lhs, Rhs>
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

impl<Lhs, Rhs> Display for SubError<Lhs, Rhs>
where
    Lhs: Display,
    Rhs: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "subtracting {} from {} would {} {}",
            self.0.rhs, self.0.lhs, self.0.kind, self.0.type_name,
        )
    }
}
