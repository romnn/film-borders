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

use std::any::Any;

pub trait NumericError: std::error::Error {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn NumericError) -> bool;
}

// impl Eq for &dyn NumericError {}

// impl PartialEq for &dyn NumericError {
//     fn eq(&self, other: &&dyn NumericError) -> bool {
//         false
//     }
// }

// impl Eq for dyn NumericError {}

// impl PartialEq for dyn NumericError {
//     fn eq(&self, other: &dyn NumericError) -> bool {
//         false
//     }
// }
impl Eq for Box<dyn NumericError> {}
// impl Eq for dyn NumericError {}

impl PartialEq for Box<dyn NumericError> {
    // impl PartialEq<Self> for dyn NumericError {
    fn eq(&self, other: &Self) -> bool {
        NumericError::eq(self.as_ref(), other.as_ref())
        // todo!();
    }
}

impl PartialEq<&Self> for Box<dyn NumericError> {
    // impl PartialEq<&Self> for dyn NumericError {
    fn eq(&self, other: &&Self) -> bool {
        NumericError::eq(self.as_ref(), other.as_ref())
        // todo!();
    }
}

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
// #[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Add(Box<dyn NumericError>),
    #[error("{0}")]
    Sub(Box<dyn NumericError>),
    #[error("{0}")]
    Mul(Box<dyn NumericError>),
    #[error("{0}")]
    Cast(Box<dyn NumericError>),
}

pub trait NumericType: std::fmt::Display + std::fmt::Debug + 'static {}

impl<T> NumericType for T where T: num::Num + std::fmt::Debug + std::fmt::Display + 'static {}

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

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub struct AddError<Lhs, Rhs>(pub ArithmeticError<Lhs, Rhs>);
// where
//     Lhs: std::fmt::Display,
//     Rhs: std::fmt::Display;

impl<Lhs, Rhs> NumericError for AddError<Lhs, Rhs>
where
    Lhs: std::fmt::Display + std::fmt::Debug + PartialEq + 'static,
    Rhs: std::fmt::Display + std::fmt::Debug + PartialEq + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &dyn NumericError) -> bool {
        match other.as_any().downcast_ref::<Self>() {
            Some(other) => PartialEq::eq(self, other),
            None => false,
        }
    }
}

impl<Lhs, Rhs> std::fmt::Display for AddError<Lhs, Rhs>
where
    Lhs: std::fmt::Display,
    Rhs: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "adding {} to {} would {} {}",
            self.0.rhs, self.0.lhs, self.0.kind, self.0.type_name,
        )
    }
}

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub struct SubError<Lhs, Rhs>(pub ArithmeticError<Lhs, Rhs>);

impl<Lhs, Rhs> NumericError for SubError<Lhs, Rhs>
where
    Lhs: std::fmt::Display + std::fmt::Debug + PartialEq + 'static,
    Rhs: std::fmt::Display + std::fmt::Debug + PartialEq + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &dyn NumericError) -> bool {
        match other.as_any().downcast_ref::<Self>() {
            Some(other) => PartialEq::eq(self, other),
            None => false,
        }
    }
}

impl<Lhs, Rhs> std::fmt::Display for SubError<Lhs, Rhs>
where
    Lhs: std::fmt::Display,
    Rhs: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "subtracting {} from {} would {} {}",
            self.0.rhs, self.0.lhs, self.0.kind, self.0.type_name,
        )
    }
}

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub struct MulError<Lhs, Rhs>(pub ArithmeticError<Lhs, Rhs>);

impl<Lhs, Rhs> NumericError for MulError<Lhs, Rhs>
where
    Lhs: std::fmt::Display + std::fmt::Debug + PartialEq + 'static,
    Rhs: std::fmt::Display + std::fmt::Debug + PartialEq + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &dyn NumericError) -> bool {
        match other.as_any().downcast_ref::<Self>() {
            Some(other) => PartialEq::eq(self, other),
            None => false,
        }
    }
}

impl<Lhs, Rhs> std::fmt::Display for MulError<Lhs, Rhs>
where
    Lhs: std::fmt::Display,
    Rhs: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "multiplying {} by {} would {} {}",
            self.0.lhs, self.0.rhs, self.0.kind, self.0.type_name,
        )
    }
}

#[derive(thiserror::Error, PartialEq, Eq)]
pub struct CastError<Src, Target> {
    pub src: Src,
    pub target: std::marker::PhantomData<Target>,
}

impl<Src, Target> From<CastError<Src, Target>> for Error
where
    Src: std::fmt::Display + std::fmt::Debug + PartialEq + 'static,
    Target: std::fmt::Display + std::fmt::Debug + PartialEq + 'static,
{
    fn from(err: CastError<Src, Target>) -> Self {
        Error::Cast(Box::new(err))
    }
}

impl<Src, Target> NumericError for CastError<Src, Target>
where
    Src: std::fmt::Display + std::fmt::Debug + PartialEq + 'static,
    Target: std::fmt::Display + std::fmt::Debug + PartialEq + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &dyn NumericError) -> bool {
        match other.as_any().downcast_ref::<Self>() {
            Some(other) => PartialEq::eq(self, other),
            None => false,
        }
    }
}

impl<Src, Target> std::fmt::Debug for CastError<Src, Target>
where
    Src: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self,)
    }
}

impl<Src, Target> std::fmt::Display for CastError<Src, Target>
where
    Src: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "cannot cast {} of type {} to {}",
            self.src,
            std::any::type_name::<Src>().to_string(),
            std::any::type_name::<Target>().to_string(),
        )
    }
}

pub trait ArithmeticOp<Lhs>
where
    Self: Sized + Clone + NumericType,
    Lhs: Sized + Clone + NumericType,
{
    fn overflows<T>(&self, lhs: &Lhs) -> ArithmeticError<Lhs, Self>
    where
        T: NumericType,
    {
        ArithmeticError {
            lhs: lhs.clone(),
            rhs: self.clone(),
            type_name: std::any::type_name::<T>().to_string(),
            kind: ArithmeticErrorKind::Overflow,
        }
    }

    fn underflows<T>(&self, lhs: &Lhs) -> ArithmeticError<Lhs, Self>
    where
        T: NumericType,
    {
        ArithmeticError {
            lhs: lhs.clone(),
            rhs: self.clone(),
            type_name: std::any::type_name::<T>().to_string(),
            kind: ArithmeticErrorKind::Underflow,
        }
    }
}

impl<L, R> ArithmeticOp<L> for R
where
    L: Sized + Clone + NumericType,
    R: Sized + Clone + NumericType,
{
}

pub trait NumCast
where
    Self: Sized + num::ToPrimitive + Copy,
{
    fn cast<Target>(self) -> Result<Target, CastError<Self, Target>>
    where
        Target: num::NumCast;
}

impl<Src> NumCast for Src
where
    Self: Sized + num::ToPrimitive + Copy,
{
    fn cast<Target>(self) -> Result<Target, CastError<Self, Target>>
    where
        Target: num::NumCast,
    {
        num::NumCast::from(self).ok_or(CastError {
            src: self,
            target: std::marker::PhantomData,
        })
    }
}

pub trait CheckedMul<Rhs = Self>
where
    Self: Sized,
{
    type Output;

    fn checked_mul(&self, scalar: &Rhs) -> Result<Self::Output, MulError<Self, Rhs>>;
}

pub trait CheckedAdd<Rhs = Self>
where
    Self: Sized,
{
    type Output;

    fn checked_add(&self, rhs: &Rhs) -> Result<Self::Output, AddError<Self, Rhs>>;
}

macro_rules! impl_unsigned_checked_add {
    ( $T:ty ) => {
        impl CheckedAdd for $T {
            type Output = $T;
            fn checked_add(&self, rhs: &Self) -> Result<Self::Output, AddError<Self, Self>> {
                num::CheckedAdd::checked_add(self, rhs)
                    .ok_or(rhs.overflows::<$T>(self))
                    .map_err(AddError)
            }
        }
    };
}

macro_rules! impl_signed_checked_add {
    ( $T:ty ) => {
        impl CheckedAdd for $T {
            type Output = $T;
            fn checked_add(&self, rhs: &Self) -> Result<Self::Output, AddError<$T, $T>> {
                if rhs.is_negative() {
                    num::CheckedSub::checked_sub(self, &rhs.abs())
                        .ok_or(rhs.underflows::<$T>(self))
                        .map_err(AddError)
                } else {
                    num::CheckedAdd::checked_add(self, &rhs)
                        .ok_or(rhs.overflows::<$T>(self))
                        .map_err(AddError)
                }
            }
        }
    };
}

// impl_unsigned_checked_add!(u32);
impl_signed_checked_add!(i64);

pub trait CheckedSub<Rhs = Self>
where
    Self: Sized,
{
    type Output;

    fn checked_sub(&self, rhs: &Rhs) -> Result<Self::Output, SubError<Self, Rhs>>;
}

macro_rules! impl_unsigned_checked_sub {
    ( $T:ty ) => {
        impl CheckedSub for $T {
            type Output = $T;
            fn checked_sub(&self, rhs: &Self) -> Result<Self::Output, SubError<Self, Self>> {
                num::CheckedSub::checked_sub(self, rhs)
                    .ok_or(rhs.underflows::<$T>(self))
                    .map_err(SubError)
            }
        }
    };
}

macro_rules! impl_signed_checked_sub {
    ( $T:ty ) => {
        impl CheckedSub for $T {
            type Output = $T;
            fn checked_sub(&self, rhs: &Self) -> Result<Self::Output, SubError<$T, $T>> {
                if rhs.is_negative() {
                    num::CheckedAdd::checked_add(self, &rhs.abs())
                        .ok_or(rhs.overflows::<$T>(self))
                        .map_err(SubError)
                } else {
                    num::CheckedSub::checked_sub(self, &rhs)
                        .ok_or(rhs.underflows::<$T>(self))
                        .map_err(SubError)
                }
            }
        }
    };
}

// impl_unsigned_checked_sub!(u32);
impl_signed_checked_sub!(i64);

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

    #[test]
    fn invalid_num_cast() {
        assert_eq!(
            &42_000f64.cast::<i8>().err().unwrap().to_string(),
            "cannot cast 42000 of type f64 to i8"
        );
        assert_eq!(
            &(-42f64).cast::<u32>().err().unwrap().to_string(),
            "cannot cast -42 of type f64 to u32"
        );
        assert_eq!(
            &(-42i64).cast::<u32>().err().unwrap().to_string(),
            "cannot cast -42 of type i64 to u32"
        );
        let value = i64::MAX;
        assert_eq!(
            &value.cast::<u32>().err().unwrap().to_string(),
            &format!("cannot cast {} of type i64 to u32", &value)
        );
        let value = i64::MIN;
        assert_eq!(
            &value.cast::<u64>().err().unwrap().to_string(),
            &format!("cannot cast {} of type i64 to u64", &value)
        );
    }

    #[test]
    fn valid_num_cast() {
        assert_eq!(42f64.cast::<f32>().ok(), Some(42f32));
        assert_eq!(42f32.cast::<f64>().ok(), Some(42f64));
        assert_eq!(42u64.cast::<f32>().ok(), Some(42f32));
        assert_eq!(42i64.cast::<f32>().ok(), Some(42f32));
        assert_eq!(42.1f64.cast::<i8>().ok(), Some(42i8));
        assert_eq!(42.6f64.cast::<i8>().ok(), Some(42i8));
        assert!(u32::MAX.cast::<i64>().is_ok());
        assert!(i64::MAX.cast::<u64>().is_ok());
        assert!(i128::MAX.cast::<f64>().is_ok());
        assert!(u128::MAX.cast::<f64>().is_ok());
        assert_eq!(f32::MAX.cast::<u32>().ok(), None);
        approx::abs_diff_eq!(u32::MAX.cast::<f32>().unwrap(), 2f32.powi(32));
        approx::abs_diff_eq!(u32::MAX.cast::<f64>().unwrap(), 2f64.powi(32),);
    }

    #[test]
    fn numeric_error_partial_eq() {
        let add_err1: Box<dyn NumericError> = Box::new(AddError(10u32.overflows::<u32>(&10u32)));
        let add_err2: Box<dyn NumericError> = Box::new(AddError(10u32.overflows::<u64>(&10u32)));
        assert!(add_err1 == add_err1);
        assert!(add_err1 != add_err2);

        let sub_err1: Box<dyn NumericError> = Box::new(SubError(10u32.overflows::<u32>(&12u32)));
        let sub_err2: Box<dyn NumericError> = Box::new(SubError(10u32.overflows::<u32>(&15u32)));
        assert!(sub_err1 == sub_err1);
        assert!(sub_err1 != sub_err2);
    }
}
