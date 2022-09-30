use crate::numeric::{error, ArithmeticOp, NumericType};
use std::any::Any;
use std::fmt::{self, Debug, Display};

pub trait CheckedAdd<Rhs = Self>
where
    Self: Sized,
{
    type Output;
    type Error;
    fn checked_add(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

macro_rules! impl_unsigned_checked_add {
    ( $T:ty ) => {
        impl CheckedAdd for $T {
            type Output = Self;
            type Error = AddError<Self, Self>;

            fn checked_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
                num::CheckedAdd::checked_add(&self, &rhs)
                    .ok_or(rhs.overflows(self))
                    .map_err(AddError)
            }
        }
    };
}

impl_unsigned_checked_add!(u32);

macro_rules! impl_signed_checked_add {
    ( $T:ty ) => {
        impl CheckedAdd for $T {
            type Output = Self;
            type Error = AddError<Self, Self>;

            fn checked_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
                if rhs.is_negative() {
                    num::CheckedSub::checked_sub(&self, &rhs.abs())
                        .ok_or(rhs.underflows(self))
                        .map_err(AddError)
                } else {
                    num::CheckedAdd::checked_add(&self, &rhs)
                        .ok_or(rhs.overflows(self))
                        .map_err(AddError)
                }
            }
        }
    };
}

impl_signed_checked_add!(i64);

#[derive(PartialEq, Eq, Debug)]
pub struct AddError<Lhs, Rhs>(pub error::ArithmeticError<Lhs, Rhs>);

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

impl<Lhs, Rhs> std::error::Error for AddError<Lhs, Rhs>
where
    Lhs: Display + Debug,
    Rhs: Display + Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0
            .cause
            .as_deref()
            .map(|cause: &dyn error::NumericError| cause.as_error())
    }
}

impl<Lhs, Rhs> Display for AddError<Lhs, Rhs>
where
    Lhs: Display,
    Rhs: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.kind {
            Some(kind) => write!(
                f,
                "adding {} to {} would {} {}",
                self.0.rhs,
                self.0.lhs,
                kind,
                std::any::type_name::<Lhs>().to_string(),
            ),
            None => write!(f, "cannot add {} to {}", self.0.rhs, self.0.lhs,),
        }
    }
}
