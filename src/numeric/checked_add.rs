use super::{error, ArithmeticOp, NumericType};
use std::any::Any;
use std::fmt::{self, Debug, Display};

pub trait CheckedAdd<Rhs = Self>
where
    Self: Sized,
{
    type Output;

    fn checked_add(&self, rhs: &Rhs) -> Result<Self::Output, AddError<Self, Rhs>>;
}

macro_rules! impl_unsigned_checked_add {
    ( $T:ty ) => {
        impl CheckedAdd for $T {
            type Output = $T;
            fn checked_add(&self, rhs: &Self) -> Result<Self::Output, AddError<Self, Self>> {
                num::CheckedAdd::checked_add(self, rhs)
                    .ok_or(rhs.overflows::<$T>(self))
                    .map_err(AddError)
            }
        }
    };
}

macro_rules! impl_signed_checked_add {
    ( $T:ty ) => {
        impl CheckedAdd for $T {
            type Output = $T;
            fn checked_add(&self, rhs: &Self) -> Result<Self::Output, AddError<$T, $T>> {
                if rhs.is_negative() {
                    num::CheckedSub::checked_sub(self, &rhs.abs())
                        .ok_or(rhs.underflows::<$T>(self))
                        .map_err(AddError)
                } else {
                    num::CheckedAdd::checked_add(self, &rhs)
                        .ok_or(rhs.overflows::<$T>(self))
                        .map_err(AddError)
                }
            }
        }
    };
}

// impl_unsigned_checked_add!(u32);
impl_signed_checked_add!(i64);

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub struct AddError<Lhs, Rhs>(pub error::ArithmeticError<Lhs, Rhs>);

impl<Lhs, Rhs> From<AddError<Lhs, Rhs>> for error::Error
where
    Lhs: NumericType,
    Rhs: NumericType,
{
    fn from(err: AddError<Lhs, Rhs>) -> Self {
        error::Error::Add(Box::new(err))
    }
}

impl<Lhs, Rhs> error::NumericError for AddError<Lhs, Rhs>
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

impl<Lhs, Rhs> Display for AddError<Lhs, Rhs>
where
    Lhs: Display,
    Rhs: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "adding {} to {} would {} {}",
            self.0.rhs, self.0.lhs, self.0.kind, self.0.type_name,
        )
    }
}
