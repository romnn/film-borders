use std::any::Any;

pub trait NumericError: std::error::Error {
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

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub enum Error {
    #[error("{0}")]
    Add(Box<dyn NumericError>),
    #[error("{0}")]
    Sub(Box<dyn NumericError>),
    #[error("{0}")]
    Mul(Box<dyn NumericError>),
    #[error("{0}")]
    Div(Box<dyn NumericError>),
    #[error("{0}")]
    Cast(Box<dyn NumericError>),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ArithmeticErrorKind {
    Overflow,
    Underflow,
}

impl std::fmt::Display for ArithmeticErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArithmeticErrorKind::Underflow => write!(f, "underflow"),
            ArithmeticErrorKind::Overflow => write!(f, "overflow"),
        }
    }
}

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub struct ArithmeticError<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
    pub type_name: String,
    pub kind: ArithmeticErrorKind,
}
