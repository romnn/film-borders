use crate::numeric::{error, NumericType};
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

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub struct MulError<Lhs, Rhs>(pub error::ArithmeticError<Lhs, Rhs>);

impl<Lhs, Rhs> From<MulError<Lhs, Rhs>> for error::Error
where
    Lhs: NumericType,
    Rhs: NumericType,
{
    fn from(err: MulError<Lhs, Rhs>) -> Self {
        error::Error::Mul(Box::new(err))
    }
}

impl<Lhs, Rhs> error::NumericError for MulError<Lhs, Rhs>
where
    Lhs: NumericType,
    Rhs: NumericType,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    // fn as_error(&self) -> &(dyn std::error::Error + 'static) {
    //     self
    // }

    fn eq(&self, other: &dyn error::NumericError) -> bool {
        match other.as_any().downcast_ref::<Self>() {
            Some(other) => PartialEq::eq(self, other),
            None => false,
        }
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
                std::any::type_name::<Lhs>().to_string(),
                // self.0.container_type_name,
            ),
            None => write!(f, "cannot multiply {} by {}", self.0.lhs, self.0.rhs,),
        }
    }
}
