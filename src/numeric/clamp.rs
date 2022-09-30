pub trait Clamp {
    fn clamp<T>(input: T, min: T, max: T) -> T
    where
        T: PartialOrd<T>;
}
