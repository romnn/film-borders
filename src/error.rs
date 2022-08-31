#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("missing output path")]
    MissingOutputFile,

    #[error("image error: `{0}`")]
    Image(#[from] image::error::ImageError),

    #[error("io error: `{0}`")]
    Io(#[from] std::io::Error),
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
