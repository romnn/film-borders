use std::any::Any;
use std::fmt::{self, Debug, Display};

pub trait AsErr {
    fn as_err(&self) -> &(dyn std::error::Error + 'static);
}

impl AsErr for dyn std::error::Error + 'static {
    fn as_err(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

impl AsErr for dyn std::error::Error + Send + 'static {
    fn as_err(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

impl AsErr for dyn std::error::Error + Sync + 'static {
    fn as_err(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

impl AsErr for dyn std::error::Error + Send + Sync + 'static {
    fn as_err(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

impl<T> AsErr for T
where
    T: std::error::Error + 'static,
{
    fn as_err(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

pub trait Arithmetic: AsErr + std::error::Error + 'static {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn Arithmetic) -> bool;
}

impl PartialEq for dyn Arithmetic + Send + Sync + 'static {
    fn eq(&self, other: &Self) -> bool {
        Arithmetic::eq(self, other)
    }
}

// required fix for derived PartialEq that otherwise moves
impl PartialEq<&Self> for Box<dyn Arithmetic + Send + Sync + 'static> {
    fn eq(&self, other: &&Self) -> bool {
        Arithmetic::eq(self.as_ref(), other.as_ref())
    }
}

#[derive(PartialEq, Debug)]
pub struct Error(pub Box<dyn Arithmetic + Sync + Send + 'static>);

impl std::ops::Deref for Error {
    type Target = dyn Arithmetic + Sync + Send + 'static;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl<E> From<E> for Error
where
    E: Arithmetic + Send + Sync,
{
    fn from(err: E) -> Self {
        Error(Box::new(err))
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
    Overflow,
    Underflow,
    DivideByZero,
}

impl Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::Underflow => write!(f, "underflow"),
            Kind::Overflow => write!(f, "overflow"),
            Kind::DivideByZero => write!(f, "divide by zero"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Operation<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
    pub kind: Option<Kind>,
    pub cause: Option<Error>,
}

pub trait DivideByZero<Rhs>
where
    Self: super::Type + Sized + Copy,
    Rhs: super::Type + Sized + Copy + num::Zero,
{
    fn divide_by_zero(self) -> Operation<Self, Rhs> {
        Operation {
            lhs: self,
            rhs: Rhs::zero(),
            kind: Some(Kind::DivideByZero),
            cause: None,
        }
    }
}

pub trait Overflow<Lhs>
where
    Self: super::Type + Sized + Copy,
    Lhs: super::Type + Sized + Copy,
{
    fn overflows(self, lhs: Lhs) -> Operation<Lhs, Self> {
        Operation {
            lhs,
            rhs: self,
            kind: Some(Kind::Overflow),
            cause: None,
        }
    }
}

pub trait Underflow<Lhs>
where
    Self: super::Type + Sized + Copy,
    Lhs: super::Type + Sized + Copy,
{
    fn underflows(self, lhs: Lhs) -> Operation<Lhs, Self> {
        Operation {
            lhs,
            rhs: self,
            kind: Some(Kind::Underflow),
            cause: None,
        }
    }
}

impl<L, R> Underflow<L> for R
where
    L: super::Type + Sized + Copy,
    R: super::Type + Sized + Copy,
{
}

impl<L, R> Overflow<L> for R
where
    L: super::Type + Sized + Copy,
    R: super::Type + Sized + Copy,
{
}

impl<R, L> DivideByZero<R> for L
where
    L: super::Type + Sized + Copy,
    R: super::Type + Sized + Copy + num::Zero,
{
}

#[cfg(test)]
mod tests {
    use super::Arithmetic as ArithmeticError;
    use crate::arithmetic::{error::Overflow, ops};

    #[test]
    fn arithmetic_error_is_std_error() {
        let err: &dyn ArithmeticError = &ops::AddError(10u32.overflows(10u32));
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn arithmetic_error_partial_eq() {
        type BoxedError = Box<dyn ArithmeticError + Send + Sync + 'static>;
        let add_err1: BoxedError = Box::new(ops::AddError(10u32.overflows(10u32)));
        let add_err2: BoxedError = Box::new(ops::AddError(10u32.overflows(10u64)));
        assert!(add_err1 == add_err1);
        assert!(add_err1 != add_err2);

        let sub_err1: BoxedError = Box::new(ops::SubError(10u32.overflows(12u32)));
        let sub_err2: BoxedError = Box::new(ops::SubError(10u32.overflows(15u32)));
        assert!(sub_err1 == sub_err1);
        assert!(sub_err1 != sub_err2);
    }
}
