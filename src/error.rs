use std::fmt::Write;

pub trait Report {
    fn report(&self) -> String;
}

impl<E> Report for E
where
    E: std::error::Error,
{
    fn report(&self) -> String {
        let mut buf = String::new();
        writeln!(buf, "ERROR: {}", self).ok();
        if let Some(cause) = self.source() {
            writeln!(buf).ok();
            writeln!(buf, "Caused by:").ok();
            let causes = std::iter::successors(Some(cause), |e| e.source());
            for (i, e) in causes.enumerate() {
                writeln!(buf, "   {}: {}", i, e).ok();
            }
        }
        buf
    }
}

#[derive(thiserror::Error, PartialEq, Debug)]
#[error("arithmetic error: {msg}")]
pub struct Arithmetic {
    pub msg: String,
    pub source: super::arithmetic::Error,
}

impl super::arithmetic::error::Arithmetic for Arithmetic {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn eq(&self, other: &dyn super::arithmetic::error::Arithmetic) -> bool {
        match other.as_any().downcast_ref::<Self>() {
            Some(other) => PartialEq::eq(self, other),
            None => false,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("missing border")]
    MissingBorder,

    #[error("missing input image")]
    MissingImage,

    #[error("image error: {0}")]
    Image(#[from] super::img::Error),

    #[error("border error: {0}")]
    Border(#[from] super::border::Error),

    #[error(transparent)]
    Arithmetic(#[from] Arithmetic),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseEnum {
    #[error("unknown enum variant: `{0}`")]
    Unknown(String),
}
