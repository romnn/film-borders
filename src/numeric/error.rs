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

pub trait Numeric: AsErr + std::error::Error + 'static {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn Numeric) -> bool;
}

// impl Eq for Box<dyn Numeric> {}
// impl Eq for Box<dyn Numeric + Send + Sync + 'static> {}
// impl Eq for dyn Numeric {}
impl Eq for dyn Numeric + Send + Sync + 'static {}

// impl PartialEq for Box<dyn Numeric> {
// impl PartialEq for dyn Numeric {
//     fn eq(&self, other: &Self) -> bool {
//         Numeric::eq(self, other)
//     }
// }

// impl PartialEq<&Self> for dyn Numeric {
//     fn eq(&self, other: &&Self) -> bool {
//         Numeric::eq(self, *other)
//     }
// }

impl PartialEq for dyn Numeric + Send + Sync + 'static {
    fn eq(&self, other: &Self) -> bool {
        Numeric::eq(self, other)
    }
}
impl PartialEq<&Self> for dyn Numeric + Send + Sync + 'static {
    fn eq(&self, other: &&Self) -> bool {
        Numeric::eq(self, *other)
    }
}

// impl PartialEq for Box<dyn Numeric + Send + Sync + 'static> {
//     fn eq(&self, other: &Self) -> bool {
//         Numeric::eq(self.as_ref(), other.as_ref())
//     }
// }

// impl PartialEq<&Self> for Box<dyn Numeric> {
//     fn eq(&self, other: &&Self) -> bool {
//         Numeric::eq(self.as_ref(), other.as_ref())
//     }
// }


impl PartialEq<&Self> for Box<dyn Numeric + Send + Sync + 'static> {
    fn eq(&self, other: &&Self) -> bool {
        Numeric::eq(self.as_ref(), other.as_ref())
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Error(pub Box<dyn Numeric + Sync + Send + 'static>);

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
    E: Numeric + Send + Sync,
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

#[derive(Debug)]
pub struct Arithmetic<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
    pub kind: Option<Kind>,
    pub cause: Option<Box<dyn Numeric + Send + Sync + 'static>>,
}

impl<Lhs, Rhs> Eq for Arithmetic<Lhs, Rhs>
where
    Lhs: Eq,
    Rhs: Eq,
{
}

impl<Lhs, Rhs> PartialEq for Arithmetic<Lhs, Rhs>
where
    Lhs: PartialEq,
    Rhs: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.rhs == other.rhs && self.kind == other.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numeric::{ops, ArithmeticOp};

    #[test]
    fn numeric_error_is_std_error() {
        let err: &dyn Numeric = &ops::AddError(10u32.overflows(10u32));
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn numeric_error_partial_eq() {
        type BoxedNumeric = Box<dyn Numeric + Send + Sync + 'static>;
        let add_err1: BoxedNumeric = Box::new(ops::AddError(10u32.overflows(10u32)));
        let add_err2: BoxedNumeric = Box::new(ops::AddError(10u32.overflows(10u64)));
        assert!(add_err1 == add_err1);
        assert!(add_err1 != add_err2);

        let sub_err1: BoxedNumeric = Box::new(ops::SubError(10u32.overflows(12u32)));
        let sub_err2: BoxedNumeric = Box::new(ops::SubError(10u32.overflows(15u32)));
        assert!(sub_err1 == sub_err1);
        assert!(sub_err1 != sub_err2);
    }
}
