use std::any::Any;
use std::fmt::{self, Debug, Display};

pub trait AsError {
    fn as_error(&self) -> &(dyn std::error::Error + 'static);
}

impl AsError for dyn std::error::Error + 'static {
    fn as_error(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

impl AsError for dyn std::error::Error + Send + 'static {
    fn as_error(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

impl AsError for dyn std::error::Error + Sync + 'static {
    fn as_error(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

impl AsError for dyn std::error::Error + Send + Sync + 'static {
    fn as_error(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

impl<T> AsError for T
where
    T: std::error::Error + 'static,
{
    fn as_error(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

pub trait NumericError: AsError + std::error::Error + 'static {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn NumericError) -> bool;
}

impl Eq for Box<dyn NumericError> {}

impl PartialEq for Box<dyn NumericError> {
    fn eq(&self, other: &Self) -> bool {
        NumericError::eq(self.as_ref(), other.as_ref())
    }
}

impl PartialEq<&Self> for Box<dyn NumericError> {
    fn eq(&self, other: &&Self) -> bool {
        NumericError::eq(self.as_ref(), other.as_ref())
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Error(pub Box<dyn NumericError>);

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
    E: NumericError,
{
    fn from(err: E) -> Self {
        Error(Box::new(err))
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ArithmeticErrorKind {
    Overflow,
    Underflow,
    DivideByZero,
}

impl Display for ArithmeticErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArithmeticErrorKind::Underflow => write!(f, "underflow"),
            ArithmeticErrorKind::Overflow => write!(f, "overflow"),
            ArithmeticErrorKind::DivideByZero => write!(f, "divide by zero"),
        }
    }
}

#[derive(Debug)]
pub struct ArithmeticError<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
    pub kind: Option<ArithmeticErrorKind>,
    pub cause: Option<Box<dyn NumericError + 'static>>,
}

impl<Lhs, Rhs> Eq for ArithmeticError<Lhs, Rhs>
where
    Lhs: Eq,
    Rhs: Eq,
{
}

impl<Lhs, Rhs> PartialEq for ArithmeticError<Lhs, Rhs>
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
        let err: &dyn NumericError = &ops::AddError(10u32.overflows(10u32));
        let std_err: &dyn std::error::Error = &err;
    }
}
