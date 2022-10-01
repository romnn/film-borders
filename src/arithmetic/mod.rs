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

pub trait Type: Display + Debug + PartialEq + Send + Sync + 'static {}

impl<T> Type for T where T: num::Num + Debug + Display + PartialEq + Send + Sync + 'static {}

#[cfg(test)]
mod tests {}
