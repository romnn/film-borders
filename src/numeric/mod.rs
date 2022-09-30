pub mod cast;
pub mod error;
pub mod ops;
pub mod option_ord;
pub mod rounding;

pub use cast::{CastError, NumCast};
pub use error::{ArithmeticError, Error};
pub use option_ord::OptionOrd;
pub use rounding::{Ceil, Floor, Round, RoundingMode};
use std::fmt::{Debug, Display};

pub trait NumericType: Display + Debug + PartialEq + 'static {}

impl<T> NumericType for T where T: num::Num + Debug + Display + PartialEq + 'static {}

pub trait ArithmeticOp<Lhs>
where
    Self: Sized + Copy + NumericType,
    Lhs: Sized + Copy + NumericType,
{
    fn overflows<T>(self, lhs: Lhs) -> error::ArithmeticError<Lhs, Self>
    where
        T: NumericType,
    {
        error::ArithmeticError {
            lhs: lhs,
            rhs: self,
            type_name: std::any::type_name::<T>().to_string(),
            kind: error::ArithmeticErrorKind::Overflow,
        }
    }

    fn underflows<T>(self, lhs: Lhs) -> error::ArithmeticError<Lhs, Self>
    where
        T: NumericType,
    {
        error::ArithmeticError {
            lhs: lhs,
            rhs: self,
            type_name: std::any::type_name::<T>().to_string(),
            kind: error::ArithmeticErrorKind::Underflow,
        }
    }
}

impl<L, R> ArithmeticOp<L> for R
where
    L: Sized + Copy + NumericType,
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
        let add_err1: Box<dyn NumericError> =
            Box::new(ops::AddError(10u32.overflows::<u32>(10u32)));
        let add_err2: Box<dyn NumericError> =
            Box::new(ops::AddError(10u32.overflows::<u64>(10u32)));
        assert!(add_err1 == add_err1);
        assert!(add_err1 != add_err2);

        let sub_err1: Box<dyn NumericError> =
            Box::new(ops::SubError(10u32.overflows::<u32>(12u32)));
        let sub_err2: Box<dyn NumericError> =
            Box::new(ops::SubError(10u32.overflows::<u32>(15u32)));
        assert!(sub_err1 == sub_err1);
        assert!(sub_err1 != sub_err2);
    }
}
