use super::types::Rect;

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
