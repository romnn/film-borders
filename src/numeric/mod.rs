pub mod cast;
pub mod clamp;
pub mod error;
pub mod ops;
pub mod option_ord;
pub mod round;

pub use cast::{Cast, CastError};
pub use error::Error;
pub use option_ord::OptionOrd;
pub use round::{Ceil, Floor, Round, RoundingMode};
use std::fmt::{Debug, Display};

pub trait Numeric: Display + Debug + PartialEq + Send + Sync + 'static {}

impl<T> Numeric for T where T: num::Num + Debug + Display + PartialEq + Send + Sync + 'static {}

pub trait ArithmeticOp<Lhs>
where
    Self: Sized + Copy + Numeric,
    Lhs: Sized + Copy + Numeric + num::Zero,
{
    fn divide_by_zero(self) -> error::Arithmetic<Self, Lhs> {
        error::Arithmetic {
            lhs: self,
            rhs: Lhs::zero(),
            kind: Some(error::Kind::DivideByZero),
            cause: None,
        }
    }

    fn overflows(self, lhs: Lhs) -> error::Arithmetic<Lhs, Self> {
        error::Arithmetic {
            lhs,
            rhs: self,
            kind: Some(error::Kind::Overflow),
            cause: None,
        }
    }

    fn underflows(self, lhs: Lhs) -> error::Arithmetic<Lhs, Self> {
        error::Arithmetic {
            lhs,
            rhs: self,
            kind: Some(error::Kind::Underflow),
            cause: None,
        }
    }
}

impl<L, R> ArithmeticOp<L> for R
where
    L: Sized + Copy + Numeric + num::Zero,
    R: Sized + Copy + Numeric,
{
}

#[cfg(test)]
mod tests {
}
