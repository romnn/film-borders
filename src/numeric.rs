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
pub enum Error {}

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

#[derive(PartialEq, Eq, Debug)]
pub struct ArithmeticError<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
    pub type_name: String,
    pub kind: ArithmeticErrorKind,
}

#[derive(PartialEq, Eq, Debug)]
pub struct AddError<Lhs, Rhs>(pub ArithmeticError<Lhs, Rhs>);

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

#[derive(PartialEq, Eq, Debug)]
pub struct SubError<Lhs, Rhs>(pub ArithmeticError<Lhs, Rhs>);

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

#[derive(PartialEq, Eq, Debug)]
pub struct MulError<Lhs, Rhs>(pub ArithmeticError<Lhs, Rhs>);

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

#[derive(PartialEq, Eq, Debug)]
pub struct CastError<Src, Target> {
    pub src: Src,
    // pub target: Target,
    pub target: std::marker::PhantomData<Target>,
    // pub type_name: String,
    // pub kind: ArithmeticErrorKind,
}

impl<Src, Target> std::fmt::Display for CastError<Src, Target>
where
    Src: std::fmt::Display,
    // Target: std::fmt::Display,
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
            type_name: std::any::type_name::<Lhs>().to_string(),
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
            type_name: std::any::type_name::<Lhs>().to_string(),
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

// pub trait NumCast<Target>
// where
//     Target: Sized,
// {
//     // : Sized + num::NumCast {
//     fn from<Src>(n: Src) -> Result<Target, CastError<Src, Target>>
//     where
//         Src: num::ToPrimitive + Copy; // num::NumCast,
//                                       //     T: num::NumCast;
// }

pub trait NumCast
where
    Self: Sized + num::ToPrimitive + Copy,
{
    // : Sized + num::NumCast {
    fn cast<Target>(self) -> Result<Target, CastError<Self, Target>>
    where
        //     Src: num::ToPrimitive + Copy; // num::NumCast,
        Target: num::NumCast;
}

// impl<U> NumCast<U> for U
impl<U> NumCast for U
// impl<T> NumCast //  for U
where
    // Self: num::ToPrimitive,
    Self: Sized + num::ToPrimitive + Copy, // num::NumCast,
                                           // U: num::NumCast,
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
    // fn from<T>(n: T) -> Result<U, CastError<T, U>>
    // where
    //     T: num::ToPrimitive + Copy, // num::NumCast,
    // {
    //     // let result: Option<Self> = num::NumCast::from(n);
    //     // Ok(result.unwrap())
    //     num::NumCast::from(n).ok_or(CastError {
    //         src: n,
    //         target: std::marker::PhantomData,
    //     })

    //     // num::cast::<T, Self>(n).ok_or(CastError {
    //     //     src: n,
    //     //     target: std::marker::PhantomData,
    //     // })
    // }
}

// pub trait NumCast<Rhs = Self>: Sized {
//     type Output;

//     fn checked_mul(&self, scalar: &Rhs) -> Result<Self::Output, MulError<Self, Rhs>>;
// }

pub trait CheckedMul<Rhs = Self>: Sized {
    type Output;

    fn checked_mul(&self, scalar: &Rhs) -> Result<Self::Output, MulError<Self, Rhs>>;
}

pub trait CheckedAdd<Rhs = Self>: Sized {
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

pub trait CheckedSub<Rhs = Self>: Sized {
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
        assert_eq!(f32::MAX.cast::<u32>().ok(), None);
        approx::abs_diff_eq!(u32::MAX.cast::<f32>().unwrap(), 2f32.powi(32));
        approx::abs_diff_eq!(u32::MAX.cast::<f64>().unwrap(), 2f64.powi(32),);
    }
}
