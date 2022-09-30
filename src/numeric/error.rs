use std::any::Any;
use std::fmt::{self, Debug, Display};

pub trait AsError {
    fn as_error(&self) -> &(dyn std::error::Error + 'static);
}

// impl<T> AsError for T
// where
//     T: NumericError,
// {
//     fn as_error(&self) -> &(dyn std::error::Error + 'static) {
//         self
//     }
// }

// impl AsError for dyn NumericError + 'static {
//     fn as_error(&self) -> &(dyn std::error::Error + 'static) {
//         self.as_
//     }
// }

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

// impl<T> AsError for Box<T>
// where
//     T: std::error::Error + 'static,
// {
//     fn as_error(&self) -> &(dyn std::error::Error + 'static) {
//         self
//     }
// }

// impl<T> AsError for T
// where
//     T: std::error::Error + 'static,
// {
//     fn as_error(&self) -> &(dyn std::error::Error + 'static) {
//         self
//     }
// }

pub trait NumericError: AsError + std::error::Error + 'static {
    fn as_any(&self) -> &dyn Any;
    // fn as_error(&self) -> &(dyn std::error::Error + 'static);
    fn eq(&self, other: &dyn NumericError) -> bool;
}

// pub trait NumericError: std::error::Error + 'static {
//     fn as_any(&self) -> &dyn Any where Self: Sized {
//         self
//     }
//     fn as_error(&self) -> &(dyn std::error::Error + 'static) {
//         self
//     }
//     fn eq(&self, other: &dyn NumericError) -> bool;
// }

// impl<Src> error::NumericError for CastError<Src>
// where
//     Src: NumericType,
// {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn as_error(&self) -> &(dyn std::error::Error + 'static) {
//         self
//     }

//     fn eq(&self, other: &dyn error::NumericError) -> bool {
//         match other.as_any().downcast_ref::<Self>() {
//             Some(other) => PartialEq::eq(self, other),
//             None => false,
//         }
//     }
// }

// impl<'a> thiserror::private::AsDynError<'a> for dyn NumericError + 'a {
//     #[inline]
//     fn as_dyn_error(&self) -> &(dyn std::error::Error + 'a) {
//         self.as_ref()
//     }
// }

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

impl Display for ArithmeticErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArithmeticErrorKind::Underflow => write!(f, "underflow"),
            ArithmeticErrorKind::Overflow => write!(f, "overflow"),
        }
    }
}

// pub trait SomeErr: std::error::Error + Send + 'static {}

// impl PartialEq for dyn SomeErr {
//     fn eq(&self, other: &dyn SomeErr) -> bool {
//         false
//     }
// }

// #[derive(thiserror::Error, Debug)]
#[derive(Debug)]
// #[derive(PartialEq, Eq, Debug)]
// #[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub struct ArithmeticError<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
    // pub container_type_name: String,
    pub kind: Option<ArithmeticErrorKind>,
    // pub source: Option<Box<dyn SomeErr>>,
    // #[source]
    pub cause: Option<Box<dyn NumericError + 'static>>,
    // pub source: Option<Box<dyn std::error::Error>>,
    // pub source: Option<Box<dyn std::error::Error + Eq + 'static>>,
    // pub source: Option<S>,
    // pub source: Box<dyn NumericError>,
    // pub source: Box<dyn NumericError>,
    // pub source: Box<dyn std::error::Error>
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

// impl<Lhs, Rhs> Display for ArithmeticError<Lhs, Rhs>
// where
//     Lhs: Display,
//     Rhs: Display,
// {
//     #[inline]
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "")
//         // write!(f, "{} {}", self.lhs, self.rhs)
//         // match self.cause {
//         //     Some(cause) => Display::fmt(cause, f),
//         //     None =>
//     }
// }

// impl<Lhs, Rhs> std::error::Error for ArithmeticError<Lhs, Rhs>
// where
//     Lhs: Debug + Display,
//     Rhs: Debug + Display,
// {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         // self.source.as_deref() as Option<&(dyn std::error::Error + 'static)>
//         // match
//         // Some(&self.source.unwrap().as_ref())
//         // use snafu::AsError;
//         // match *self {
//         //     Self { ref cause, .. } => Some(cause.as_error()),
//         // }
//         // match self.cause {
//         //     Some(cause) => Display::fmt(cause, f),
//         //     None =>

//         // self.cause
//         //     .as_deref()
//         //     .map(|cause: &dyn NumericError| cause.as_error())
//         // .map(|err: &dyn NumericError| &err as &dyn std::error::Error)
//         match self.cause {
//             Some(ref cause) => {
//                 Some(cause.as_ref().as_error())
//                 // let test:  = cause.as_ref().clone();
//                 // Some(test.as_error())
//             }
//             None => None,
//         }
//         // let err: &dyn NumericError = self.source.unwrap().as_ref();
//         // let std_err: &dyn std::error::Error = &err;
//         // Some(std_err)

//         // Some(self.source.unwrap().as_ref())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numeric::{ops, ArithmeticOp};

    #[test]
    fn thiserror_source() {
        #[derive(thiserror::Error, Debug)]
        pub struct TestError {
            pub source: Option<Box<dyn std::error::Error>>,
        }
        impl std::fmt::Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "this is a test")
            }
        }
    }

    #[test]
    fn numeric_error_is_std_error() {
        let err: &dyn NumericError = &ops::AddError(10u32.overflows::<u32>(10u32));
        let std_err: &dyn std::error::Error = &err;
        // let err: Box<dyn NumericError> = Box::new(ops::AddError(10u32.overflows::<u32>(10u32)));
        // let std_err: Box<dyn std::error::Error> = err;
    }
}
