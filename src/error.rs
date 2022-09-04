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
    Border(#[from] BorderError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum BorderError {
    #[error("border must contain at least one transparent area, found {0:?}")]
    BadTransparency(Vec<Rect>),

    #[error("invalid border: {0}")]
    Invalid(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ColorError {
    #[error("invalid hex color: `{0}`")]
    InvalidHex(String),

    #[error("hex color is missing component")]
    MissingComponent,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseEnumError {
    #[error("unknown enum variant: `{0}`")]
    Unknown(String),
}
