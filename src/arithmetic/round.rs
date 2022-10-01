pub trait RoundingMode {
    fn round<F>(value: F) -> F
    where
        F: num::Float;
}

pub struct Ceil {}
pub struct Floor {}
pub struct Round {}

impl RoundingMode for Round {
    fn round<F>(value: F) -> F
    where
        F: num::Float,
    {
        value.round()
    }
}

impl RoundingMode for Ceil {
    fn round<F>(value: F) -> F
    where
        F: num::Float,
    {
        value.ceil()
    }
}

impl RoundingMode for Floor {
    fn round<F>(value: F) -> F
    where
        F: num::Float,
    {
        value.floor()
    }
}
