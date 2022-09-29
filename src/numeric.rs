use num::traits::Float;
use std::cmp::Ordering;
use std::fmt::Debug;

pub trait RoundingMode {
    fn round<F: Float>(value: F) -> F;
}

pub struct Ceil {}
pub struct Floor {}
pub struct Round {}

impl RoundingMode for Round {
    fn round<F: Float>(value: F) -> F {
        value.round()
    }
}

impl RoundingMode for Ceil {
    fn round<F: Float>(value: F) -> F {
        value.ceil()
    }
}

impl RoundingMode for Floor {
    fn round<F: Float>(value: F) -> F {
        value.floor()
    }
}

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub enum Error {
    Add(ArithmeticError),
    // #[error("failed to add: {0}")]
    // #[error("adding {right} to {left} would overflow")]
    // Add {
    //     left: Box<dyn NumericType>,
    //     right: Box<dyn NumericType>,
    // },
    // Add(WouldOverflowError),
    // Add(WouldOverflowError<L, R>),
    // #[error("failed to subtract: {0}")]
    // Sub(SubError<L, R>),
    // #[error("failed to subtract: {0}")]
    // Sub(#[from] SubError<L, R>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Add(err) => {}
        }
        write!(f, "")
        // write!(f, "{}x{}", self.width, self.height)
        // f.debug_struct("Point")
        //     .field("x", &self.x)
        //     .field("y", &self.y)
        //     .finish()
    }
}

// pub trait Number: num::traits::cast::AsPrimitive + Debug {}
// pub trait NumericType: std::fmt::Display + 'static {}
pub trait NumericType: std::fmt::Display + std::fmt::Debug + 'static {}

// impl<T> NumericType for T where T: num::Num + std::fmt::Display + 'static {
impl<T> NumericType for T where T: num::Num + std::fmt::Debug + std::fmt::Display + 'static {}

// #[derive(thiserror::Error, Debug)]
#[derive(Debug)]
// #[error("adding {right:?} to {left:?} would overflow")]
// // pub struct WouldOverflowError<L, R> {
pub enum ArithmeticErrorKind {
    Overflow,
    Underflow,
}

#[derive(Debug)]
pub struct ArithmeticError {
    lhs: Box<dyn NumericType>,
    rhs: Box<dyn NumericType>,
    type_id: std::any::TypeId,
    kind: ArithmeticErrorKind,
}

// impl Debug for WouldOverflowError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         f.debug_struct("WouldOverflowError")
//             .field("left", &self.left)
//             .field("right", &self.right)
//             .finish()
//     }
// }

impl Eq for ArithmeticError {}

impl PartialEq for ArithmeticError {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

pub trait ArithmeticOp<Lhs>
where
    Self: Sized + Clone + NumericType, // Sized + Debug + Clone + 'static,
    Lhs: Sized + Clone + NumericType,
{
    fn overflows<T>(&self, lhs: &Lhs) -> ArithmeticError
    where
        T: NumericType,
    {
        ArithmeticError {
            lhs: Box::new(lhs.clone()),
            rhs: Box::new(self.clone()),
            type_id: std::any::TypeId::of::<T>(),
            kind: ArithmeticErrorKind::Overflow,
        }
    }

    fn underflows<T>(&self, lhs: &Lhs) -> ArithmeticError
    where
        T: NumericType,
    {
        ArithmeticError {
            lhs: Box::new(lhs.clone()),
            rhs: Box::new(self.clone()),
            type_id: std::any::TypeId::of::<T>(),
            kind: ArithmeticErrorKind::Underflow,
        }
    }
}

impl<L, R> ArithmeticOp<L> for R
where
    L: Sized + Clone + NumericType, // Sized + Debug + Clone + 'static,
    R: Sized + Clone + NumericType, // Sized + Debug + Clone + 'static,
{
}

// impl WouldOverflowError {

// }

// #[derive(thiserror::Error, Debug)]
// pub enum AddError<L, R> where L: Sized, R: Sized {
//     #[error("adding {right} to {left} would overflow")]
//     WouldOverflow { left: L, right: R },
// }

// #[derive(thiserror::Error, PartialEq, Eq, Debug, Clone)]
// pub enum SubError<L, R> {
//     #[error("subtracting {left} from {right} would underflow")]
//     WouldOverflow { left: L, right: R },
// }

// WouldUnderflow(#[from] image::error::ImageError),
// : Sized + Add<Self, Output = Self>
pub trait CheckedAdd<Rhs = Self>: Sized {
    type Output;

    // fn checked_add(&self, v: &Rhs) -> Result<Self::Output, AddError<Self, Rhs>>;
    fn checked_add(&self, rhs: &Rhs) -> Result<Self::Output, Error>;
}

// trait TestTrait {
//     fn signed(&self) -> bool;
// }

// impl<T> TestTrait for T
// where
//     T: Signed,
// {
//     fn signed(&self) -> bool {
//         true
//     }
// }

// impl<T> TestTrait for T
// where
//     T: Unsigned,
// {
//     fn signed(&self) -> bool {
//         false
//     }
// }

// trait Signed {} // : num::Signed {}
// impl Signed for i128 {}
// // impl Signed for isize {}
// // impl Signed for i64 {}
// // impl Signed for i32 {}
// // impl Signed for i16 {}
// // impl Signed for i8 {}

// trait Unsigned {} // : num::Unsigned {}
// impl Unsigned for u128 {}
// // impl Unsigned for usize {}
// // impl Unsigned for u64 {}
// // impl Unsigned for u32 {}
// // impl Unsigned for u16 {}
// // impl Unsigned for u8 {}

macro_rules! impl_unsigned_checked_add {
    ( $T:ty ) => {
        impl CheckedAdd for $T {
            type Output = $T;
            fn checked_add(&self, rhs: &Self) -> Result<Self::Output, Error> {
                num::CheckedAdd::checked_add(self, rhs).ok_or(Error::Add(rhs.overflows::<$T>(self)))
                // if rhs.is_negative() {
                //     self.checked_sub(&rhs.abs()).ok_or(rhs.underflows::<T>(self))
                // } else {
                //     self.checked_add(rhs).ok_or(rhs.overflows::<T>(self))
                // }.map_err(Error::Add)
            }
        }
    };
}

macro_rules! impl_signed_checked_add {
    ( $T:ty ) => {
        impl CheckedAdd for $T {
            type Output = $T;
            fn checked_add(&self, rhs: &Self) -> Result<Self::Output, Error> {
                if rhs.is_negative() {
                    num::CheckedSub::checked_sub(self, &rhs.abs()).ok_or(rhs.underflows::<$T>(self))
                } else {
                    num::CheckedAdd::checked_add(self, &rhs).ok_or(rhs.overflows::<$T>(self))
                }
                .map_err(Error::Add)
            }
        }
    };
}

impl_unsigned_checked_add!(u32);
impl_signed_checked_add!(i64);

// impl<T> CheckedAdd<T> for T
// where
//     T: Signed, // Clone + NumericType + num::Signed + num::CheckedAdd + num::CheckedSub,
//                // T::Rhs: Signed,
// {
//     type Output = T;

//     fn checked_add(&self, rhs: &T) -> Result<Self::Output, Error> {
//         self.checked_add(rhs)
//             .ok_or(Error::Add(rhs.overflows::<T>(self)))
//         // if rhs.is_negative() {
//         //     self.checked_sub(&rhs.abs()).ok_or(rhs.underflows::<T>(self))
//         // } else {
//         //     self.checked_add(rhs).ok_or(rhs.overflows::<T>(self))
//         // }.map_err(Error::Add)
//     }
// }

// impl<T> CheckedAdd<T> for T
// where
//     T: Unsigned, // Clone + NumericType + num::Unsigned + num::CheckedAdd,
//                  // T::Rhs: Unsigned
// {
//     type Output = T;

//     fn checked_add(&self, rhs: &T) -> Result<Self::Output, Error> {
//         self.checked_add(rhs)
//             .ok_or(Error::Add(rhs.overflows::<T>(self)))
//         // if rhs.is_negative() {
//         //     self.checked_sub(&rhs.abs()).ok_or(rhs.underflows::<T>(self))
//         // } else {
//         //     self.checked_add(rhs).ok_or(rhs.overflows::<T>(self))
//         // }.map_err(Error::Add)
//     }
// }

// pub trait CheckedAdd: Sized + Add<Self, Output = Self> {
//     fn checked_add(&self, v: &Self) -> Option<Self>;
// }

pub trait OptionOrd {
    fn cmp(&self, other: &Self) -> Ordering;
    fn min(self, other: Self) -> Self
    where
        Self: Sized;
}

impl<T> OptionOrd for Option<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Some(v) => match other {
                Some(other) => Ord::cmp(&v, &other),
                None => Ordering::Less,
            },
            None => Ordering::Less,
        }
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match OptionOrd::cmp(&self, &other) {
            Ordering::Less | Ordering::Equal => self,
            Ordering::Greater => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn option_ord() {
        assert_eq!(OptionOrd::min(Some(10), Some(15)), Some(10));
        assert_eq!(OptionOrd::min(None::<u32>, Some(15)), None);
        assert_eq!(OptionOrd::min(None::<u32>, None), None);
        assert_eq!(OptionOrd::min(Some(10), None), Some(10));
    }
}
