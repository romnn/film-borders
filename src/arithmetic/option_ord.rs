use std::cmp::Ordering;

pub trait OptionOrd {
    fn cmp(&self, other: &Self) -> Ordering;
    #[must_use]
    fn min(self, other: Self) -> Self
    where
        Self: Sized;
}

impl<T> OptionOrd for Option<T>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Some(v) => match other {
                Some(other) => Ord::cmp(&v, &other),
                None => Ordering::Less,
            },
            None => Ordering::Less,
        }
    }

    #[inline]
    #[must_use]
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
