use crate::arithmetic;
use std::any::Any;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;

pub trait Cast
where
    Self: Sized + num::ToPrimitive + Copy,
{
    fn cast<Target>(self) -> Result<Target, CastError<Self, Target>>
    where
        Target: num::NumCast;
}

impl<Src> Cast for Src
where
    Self: Sized + num::ToPrimitive + Copy,
{
    fn cast<Target>(self) -> Result<Target, CastError<Self, Target>>
    where
        Target: num::NumCast,
    {
        num::NumCast::from(self).ok_or(CastError {
            src: self,
            target: PhantomData,
            cause: None,
        })
    }
}

#[derive(PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct CastError<Src, Target> {
    pub src: Src,
    pub target: PhantomData<Target>,
    pub cause: Option<Box<dyn arithmetic::error::Arithmetic + Send + Sync + 'static>>,
}

impl<Src, Target> arithmetic::error::Arithmetic for CastError<Src, Target>
where
    Src: arithmetic::Type,
    Target: arithmetic::Type,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &dyn arithmetic::error::Arithmetic) -> bool {
        match other.as_any().downcast_ref::<Self>() {
            Some(other) => PartialEq::eq(self, other),
            None => false,
        }
    }
}

impl<Src, Target> std::error::Error for CastError<Src, Target>
where
    Src: Debug + Display,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_deref().map(arithmetic::error::AsErr::as_err)
    }
}

impl<Src, Target> Debug for CastError<Src, Target>
where
    Src: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CastError")
            .field("src", &self.src)
            .field("target", &std::any::type_name::<Target>())
            .field("cause", &self.cause)
            .finish()
    }
}

impl<Src, Target> Display for CastError<Src, Target>
where
    Src: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "cannot cast {} of type {} to {}",
            self.src,
            std::any::type_name::<Src>(),
            std::any::type_name::<Target>(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use pretty_assertions::assert_eq;

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

        assert_abs_diff_eq!(
            u32::MAX.cast::<f32>().unwrap(),
            2f32.powi(32),
            epsilon = 2.0
        );
        assert_abs_diff_eq!(
            u32::MAX.cast::<f64>().unwrap(),
            2f64.powi(32),
            epsilon = 2.0
        );
    }
}
