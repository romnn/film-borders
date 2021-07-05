use image::error::{ImageError, ImageResult};
use image::io::Reader as ImageReader;
use image::DynamicImage;
use std::env;
use std::io::{Error as IOError, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ImageBorders {
    pub image_path: PathBuf,
    pub output_path: PathBuf,
    img: DynamicImage,
}

impl ImageBorders {
    pub fn new(image_path: PathBuf) -> Result<ImageBorders, ImageError> {
        let img = ImageReader::open(image_path.to_owned())?.decode()?;
        let default_output = env::current_dir()?.join(format!(
            "{}_with_border.jpg",
            image_path
                .file_stem()
                .ok_or_else(|| ImageError::IoError(IOError::new(ErrorKind::Other, "nooo")))?
                .to_str()
                .ok_or_else(|| ImageError::IoError(IOError::new(ErrorKind::Other, "nooo")))?
        ));
        Ok(ImageBorders {
            image_path: image_path,
            output_path: default_output,
            img: img.clone(),
        })
    }

    pub fn save_result(&self) -> Result<(), ImageError> {
        self.img.save(&self.output_path)?;
        Ok(())
    }

    pub fn apply(&self) -> Result<DynamicImage, ImageError> {
        // add another image here and
        Ok(self.img.clone())
    }
}
