pub mod cast;
pub mod clamp;
pub mod error;
pub mod ops;
pub mod option_ord;
pub mod round;

pub use cast::{CastError, NumCast};
// pub use clamp::{Clamp, ClampMax, ClampMin};
pub use error::{ArithmeticError, Error};
pub use option_ord::OptionOrd;
pub use round::{Ceil, Floor, Round, RoundingMode};
use std::fmt::{Debug, Display};

pub trait NumericType: Display + Debug + PartialEq + 'static {}

impl<T> NumericType for T where T: num::Num + Debug + Display + PartialEq + 'static {}

pub trait ArithmeticOp<Lhs>
where
    Self: Sized + Copy + NumericType,
    Lhs: Sized + Copy + NumericType + num::Zero,
{
    fn divide_by_zero(self) -> error::ArithmeticError<Self, Lhs> {
        error::ArithmeticError {
            lhs: self,
            rhs: Lhs::zero(),
            kind: Some(error::ArithmeticErrorKind::DivideByZero),
            cause: None,
        }
    }

    fn overflows(self, lhs: Lhs) -> error::ArithmeticError<Lhs, Self> {
        error::ArithmeticError {
            lhs: lhs,
            rhs: self,
            kind: Some(error::ArithmeticErrorKind::Overflow),
            cause: None,
        }
    }

    fn underflows(self, lhs: Lhs) -> error::ArithmeticError<Lhs, Self> {
        error::ArithmeticError {
            lhs: lhs,
            rhs: self,
            kind: Some(error::ArithmeticErrorKind::Underflow),
            cause: None,
        }
    }
}

impl<L, R> ArithmeticOp<L> for R
where
    L: Sized + Copy + NumericType + num::Zero,
    R: Sized + Copy + NumericType,
{
}

#[cfg(test)]
mod tests {
    use super::error::NumericError;
    use super::ops;
    use super::*;

    #[test]
    fn numeric_error_partial_eq() {
        let add_err1: Box<dyn NumericError> = Box::new(ops::AddError(10u32.overflows(10u32)));
        let add_err2: Box<dyn NumericError> = Box::new(ops::AddError(10u32.overflows(10u64)));
        assert!(add_err1 == add_err1);
        assert!(add_err1 != add_err2);

        let sub_err1: Box<dyn NumericError> = Box::new(ops::SubError(10u32.overflows(12u32)));
        let sub_err2: Box<dyn NumericError> = Box::new(ops::SubError(10u32.overflows(15u32)));
        assert!(sub_err1 == sub_err1);
        assert!(sub_err1 != sub_err2);
    }
}
