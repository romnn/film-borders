pub trait ClampMin {
    #[must_use]
    fn clamp_min<MIN>(self, min: MIN) -> Self
    where
        Self: Sized,
        MIN: Into<Self>;
}

pub trait Clamp {
    // fn clamp<T>(value: T, min: T, max: T) -> T;
    #[must_use]
    fn clamp<MIN, MAX>(self, min: MIN, max: MAX) -> Self
    where
        Self: Sized,
        MIN: Into<Self>,
        MAX: Into<Self>;
}

impl<T> Clamp for T
where
    Self: Sized + PartialOrd<Self>,
{
    #[inline]
    #[must_use]
    fn clamp<MIN, MAX>(self, min: MIN, max: MAX) -> Self
    where
        MIN: Into<Self>,
        MAX: Into<Self>,
    {
        num::clamp(self, min.into(), max.into())
    }
}

impl<T> ClampMin for T
where
    Self: Sized + PartialOrd<Self>,
{
    #[inline]
    #[must_use]
    fn clamp_min<MIN>(self, min: MIN) -> Self
    where
        MIN: Into<Self>,
    {
        num::traits::clamp_min(self, min.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{Clamp, ClampMin};
    use pretty_assertions::assert_eq;

    #[test]
    fn clamp_using_partial_ord() {
        assert_eq!(Clamp::clamp(10u32, 5u32, 12u32), 10u32);
        assert_eq!(Clamp::clamp(10u32, 11u32, 12u32), 11u32);
        assert_eq!(Clamp::clamp(3f32, 0f32, 2f32), 2f32);
    }

    #[test]
    fn clamp_min_using_partial_ord() {
        assert_eq!(ClampMin::clamp_min(10u32, 5u32), 10u32);
        assert_eq!(ClampMin::clamp_min(10u32, 11u32), 11u32);
        assert_eq!(ClampMin::clamp_min(3f32, 0f32), 3f32);
    }
}
