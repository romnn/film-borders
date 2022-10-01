pub mod cast;
pub mod clamp;
pub mod error;
pub mod ops;
pub mod option_ord;
pub mod round;

pub use cast::{CastError, Cast};
pub use error::Error;
pub use option_ord::OptionOrd;
pub use round::{Ceil, Floor, Round, RoundingMode};
use std::fmt::{Debug, Display};

pub trait Numeric: Display + Debug + PartialEq + 'static {}

impl<T> Numeric for T where T: num::Num + Debug + Display + PartialEq + 'static {}

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
    use super::error;
    use super::ops;
    use super::*;

    #[test]
    fn numeric_error_partial_eq() {
        let add_err1: Box<dyn error::Numeric> = Box::new(ops::AddError(10u32.overflows(10u32)));
        let add_err2: Box<dyn error::Numeric> = Box::new(ops::AddError(10u32.overflows(10u64)));
        assert!(add_err1 == add_err1);
        assert!(add_err1 != add_err2);

        let sub_err1: Box<dyn error::Numeric> = Box::new(ops::SubError(10u32.overflows(12u32)));
        let sub_err2: Box<dyn error::Numeric> = Box::new(ops::SubError(10u32.overflows(15u32)));
        assert!(sub_err1 == sub_err1);
        assert!(sub_err1 != sub_err2);
    }
}
