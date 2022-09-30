use super::types::Rect;
use std::fmt;
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
        write!(buf, "ERROR: {}", self.to_string());
        if let Some(cause) = self.source() {
            write!(buf, "\n");
            write!(buf, "Caused by:\n");
            let causes = std::iter::successors(Some(cause), |e| e.source());
            for (i, e) in causes.enumerate() {
                write!(buf, "   {}: {}\n", i, e);
            }
        }
        buf
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("missing border")]
    MissingBorder,

    #[error("missing input image")]
    MissingImage,

    #[error("missing output path")]
    MissingOutputFile,

    #[error("image error: {0}")]
    Image(#[from] image::error::ImageError),

    #[error("border error: {0}")]
    Border(#[from] super::border::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseEnumError {
    #[error("unknown enum variant: `{0}`")]
    Unknown(String),
}
