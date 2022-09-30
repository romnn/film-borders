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
