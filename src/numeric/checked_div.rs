use super::{error, NumericType};
use std::any::Any;
use std::fmt::{self, Debug, Display};

pub trait CheckedDiv<Rhs = Self>
where
    Self: Sized,
{
    type Output;
    type Error;

    fn checked_div(&self, scalar: Rhs) -> Result<Self::Output, Self::Error>;
}

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub struct DivError<Lhs, Rhs>(pub error::ArithmeticError<Lhs, Rhs>);

impl<Lhs, Rhs> From<DivError<Lhs, Rhs>> for error::Error
where
    Lhs: NumericType,
    Rhs: NumericType,
{
    fn from(err: DivError<Lhs, Rhs>) -> Self {
        error::Error::Mul(Box::new(err))
    }
}

impl<Lhs, Rhs> error::NumericError for DivError<Lhs, Rhs>
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

impl<Lhs, Rhs> Display for DivError<Lhs, Rhs>
where
    Lhs: Display,
    Rhs: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "dividing {} by {} would {} {}",
            self.0.lhs, self.0.rhs, self.0.kind, self.0.type_name,
        )
    }
}
