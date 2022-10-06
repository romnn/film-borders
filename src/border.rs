use super::arithmetic::{ops::CheckedSub, Round};
use super::img::{self, Image};
use super::types::{self, Point, Rect, Size};
use super::{arithmetic, debug, error, imageops};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub transparent_component_threshold: u32,
    pub alpha_threshold: f64,
}

impl Default for Options {
    #[inline]
    fn default() -> Self {
        Self {
            transparent_component_threshold: 8,
            alpha_threshold: 0.95,
        }
    }
}

pub enum Kind {
    #[cfg(feature = "builtin")]
    Builtin(super::builtin::Builtin),
    Custom(Border),
}

impl Kind {
    #[inline]
    pub fn into_border(self) -> Result<Border, Error> {
        match self {
            #[cfg(feature = "builtin")]
            Self::Builtin(builtin) => builtin.into_border(),
            Self::Custom(border) => Ok(border),
        }
    }
}

impl std::fmt::Debug for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            #[cfg(feature = "builtin")]
            Kind::Builtin(builtin) => write!(f, "Builtin({:?})", builtin),
            Kind::Custom(_) => write!(f, "Custom"),
        }
    }
}

#[derive(Clone)]
pub struct Border {
    inner: Image,
    options: Option<Options>,
    transparent_components: Vec<Rect>,
}

impl std::ops::Deref for Border {
    type Target = Image;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Border {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Border {
    #[inline]
    pub fn from_reader<R: std::io::BufRead + std::io::Seek>(
        reader: R,
        options: Option<Options>,
    ) -> Result<Self, Error> {
        let image = Image::from_reader(reader).map_err(img::Error::from)?;
        Self::from_image(image, options).map_err(Error::from)
    }

    #[inline]
    pub fn open(path: impl Into<PathBuf>, options: Option<Options>) -> Result<Self, Error> {
        let image = Image::open(path.into()).map_err(img::Error::from)?;
        Self::from_image(image, options).map_err(Error::from)
    }

    #[inline]
    pub fn from_image(
        inner: Image,
        options: Option<Options>,
    ) -> Result<Self, TransparentComponentsError> {
        let mut border = Self {
            inner,
            options,
            transparent_components: Vec::new(),
        };
        border.compute_transparent_components(options)?;
        Ok(border)
    }

    #[inline]
    pub fn custom(
        mut border: Self,
        content_size: Size,
        stich_direction: Option<types::Orientation>,
    ) -> Result<Self, Error> {
        use arithmetic::{
            ops::{CheckedAdd, CheckedDiv, CheckedMul},
            Cast,
        };

        let components = border.transparent_components().to_vec();
        if components.len() != 1 {
            return Err(Error::Invalid(InvalidTransparentComponentsError {
                required: (Ordering::Equal, 1),
                components,
            }));
        }
        // by default, use the longer dimension to stich
        let stich_direction = stich_direction.unwrap_or(types::Orientation::Portrait);
        border.rotate_to_orientation(stich_direction)?;
        let original_orientation = content_size.orientation();
        let content_size = content_size.rotate_to_orientation(stich_direction);
        debug!(&content_size);

        let border_size = border.size_for(types::BoundedSize {
            width: Some(content_size.width),
            height: None,
        })?;
        debug!(&border_size);
        border.resize_and_crop(border_size, types::ResizeMode::Cover)?;

        // border is portrait now, we stich vertically
        // todo: find optimal overlay patches somehow
        let (top_patch_rect, top_patch_size) =
            compute_patch_rect(border_size, 0.0, 0.25).map_err(|err| error::Arithmetic {
                msg: "failed to compute top patch".into(),
                source: err,
            })?;

        let (bottom_patch_rect, bottom_patch_size) = compute_patch_rect(border_size, 0.75, 1.0)
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute bottom patch".into(),
                source: err,
            })?;

        let (overlay_patch_rect, overlay_patch_size) = compute_patch_rect(border_size, 0.3, 0.7)
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute overlay patch rect".into(),
                source: err,
            })?;

        // create buffer for the new border
        let border_content_size = border.content_size()?;
        debug!(&border_content_size);

        let border_padding = border_size
            .checked_sub(border_content_size)
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute border padding".to_string(),
                source: err.into(),
            })?;
        debug!(&border_padding);

        let new_border_size =
            content_size
                .checked_add(border_padding)
                .map_err(|err| error::Arithmetic {
                    msg: "failed to compute new border size".to_string(),
                    source: err.into(),
                })?;

        let mut new_border = Image::with_size(new_border_size);
        debug!(&new_border.size());

        #[cfg(feature = "debug")]
        {
            let green = types::Color::rgba(0, 255, 0, 255);
            let clear = types::Color::clear();
            new_border
                .fill(green, imageops::FillMode::Set)
                .map_err(img::Error::from)?;

            let content_rect = border.content_rect()?;
            debug!(&content_rect);

            let new_border_content_rect = (|| {
                let bottom_right_padding =
                    Point::from(border.size()).checked_sub(content_rect.bottom_right())?;
                let new_border_content_rect_bottom_right =
                    Point::from(new_border_size).checked_sub(bottom_right_padding)?;

                let rect = Rect::from_points(
                    content_rect.top_left(),
                    new_border_content_rect_bottom_right,
                );
                Ok::<_, arithmetic::Error>(rect)
            })();
            let new_border_content_rect =
                new_border_content_rect.map_err(|err| error::Arithmetic {
                    msg: "failed to compute new border content rect".into(),
                    source: err,
                })?;

            new_border
                .fill_rect(clear, &new_border_content_rect, imageops::FillMode::Set)
                .map_err(img::Error::from)?;
        }

        // draw top patch
        let mut border_top: Image = border.inner.clone();
        #[cfg(feature = "debug")]
        border_top
            .clip_alpha(&border_top.size().into(), 0, 200)
            .map_err(img::Error::from)?;
        border_top.crop(&top_patch_rect);
        new_border.overlay(&border_top, Point::origin());

        // draw bottom patch
        let mut border_bottom = border.inner.clone();
        #[cfg(feature = "debug")]
        border_bottom
            .clip_alpha(&border_bottom.size().into(), 0, 200)
            .map_err(img::Error::from)?;
        border_bottom.crop(&bottom_patch_rect);

        let bottom_patch_top_left = Point::from(new_border_size)
            .checked_sub(bottom_patch_size.into())
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute bottom patch top left".to_string(),
                source: err.into(),
            })?;
        new_border.overlay(&border_bottom, bottom_patch_top_left);

        // draw patches in between
        let fill_height = (|| {
            let mut height = i64::from(new_border_size.height);
            height = CheckedSub::checked_sub(height, bottom_patch_rect.height())?;
            height = CheckedSub::checked_sub(height, top_patch_rect.height())?;
            let height = height.cast::<u32>()?;
            Ok::<_, arithmetic::Error>(height)
        })();
        let fill_height: u32 = fill_height.map_err(|err| error::Arithmetic {
            msg: "failed to compute fill height".to_string(),
            source: err,
        })?;
        debug!(&fill_height);
        debug!(&overlay_patch_size.height);

        let fade_height = (|| {
            let height = f64::from(overlay_patch_size.height)
                .checked_mul(0.2)?
                .ceil()
                .cast::<u32>()?;
            Ok::<_, arithmetic::Error>(height)
        })();
        let fade_height = fade_height.map_err(|err| error::Arithmetic {
            msg: "failed to compute fade height".to_string(),
            source: err,
        })?;
        debug!(&fade_height);

        let safe_patch_height = (|| {
            let total_fade_height = CheckedMul::checked_mul(fade_height, 2)?;
            let height = CheckedSub::checked_sub(overlay_patch_size.height, total_fade_height)?;
            Ok::<_, arithmetic::Error>(height)
        })();
        let safe_patch_height = safe_patch_height.map_err(|err| error::Arithmetic {
            msg: "failed to compute safe patch height".to_string(),
            source: err,
        })?;
        debug!(&safe_patch_height);

        let num_patches = (|| {
            let patches = f64::from(fill_height)
                .checked_div(f64::from(safe_patch_height))?
                .ceil()
                .cast::<u32>()?;
            Ok::<_, arithmetic::Error>(patches)
        })();
        let num_patches = num_patches.map_err(|err| error::Arithmetic {
            msg: "failed to compute number of patches".to_string(),
            source: err,
        })?;
        debug!(&num_patches);

        let new_safe_patch_height = (|| {
            let height = f64::from(fill_height)
                .checked_div(f64::from(num_patches))?
                .ceil()
                .cast::<u32>()?;
            Ok::<_, arithmetic::Error>(height)
        })();
        let new_safe_patch_height = new_safe_patch_height.map_err(|err| error::Arithmetic {
            msg: "failed to compute new safe patch height".to_string(),
            source: err,
        })?;
        debug!(&new_safe_patch_height);
        assert!(new_safe_patch_height <= safe_patch_height);

        let patch_height = (|| {
            let total_fade_height = CheckedMul::checked_mul(fade_height, 2)?;
            let height = CheckedAdd::checked_add(safe_patch_height, total_fade_height)?;
            Ok::<_, arithmetic::Error>(height)
        })();
        let patch_height = patch_height.map_err(|err| error::Arithmetic {
            msg: "failed to compute patch height".to_string(),
            source: err,
        })?;
        debug!(&patch_height);

        let patch_size = Size {
            width: overlay_patch_size.width,
            height: patch_height,
        };

        for i in 0..num_patches {
            let patch_top_left = (|| {
                let mut patch_offset_y =
                    CheckedMul::checked_mul(i64::from(i), i64::from(safe_patch_height))?;
                patch_offset_y = CheckedSub::checked_sub(patch_offset_y, i64::from(fade_height))?;

                let top_left = top_patch_rect.bottom_left().checked_add(Point {
                    x: 0,
                    y: patch_offset_y,
                })?;
                Ok::<_, arithmetic::Error>(top_left)
            })();

            let patch_top_left = patch_top_left.map_err(|err| error::Arithmetic {
                msg: "failed to compute patch top left".to_string(),
                source: err,
            })?;

            overlay_and_fade_patch(
                &mut new_border,
                border.inner.clone(),
                patch_top_left,
                patch_size,
                fade_height,
            )?;
        }

        let mut new_border = Self::from_image(new_border, border.options)?;
        new_border.rotate_to_orientation(original_orientation)?;
        Ok(new_border)
    }

    #[inline]
    fn compute_transparent_components(
        &mut self,
        options: Option<Options>,
    ) -> Result<(), TransparentComponentsError> {
        let options = options.unwrap_or_default();
        self.transparent_components = imageops::find_transparent_components(
            &self.inner,
            options.alpha_threshold,
            options.transparent_component_threshold,
        )?;

        if self.transparent_components.is_empty() {
            return Err(TransparentComponentsError::Invalid(
                InvalidTransparentComponentsError {
                    required: (Ordering::Greater, 0),
                    components: self.transparent_components.clone(),
                },
            ));
        }
        self.transparent_components
            .sort_by_key(|b| std::cmp::Reverse(b.pixel_count().unwrap_or(0)));
        Ok(())
    }

    #[inline]
    pub fn resize_and_crop(
        &mut self,
        container: Size,
        resize_mode: types::ResizeMode,
    ) -> Result<(), Error> {
        let crop_mode = super::CropMode::Center;
        self.inner
            .resize_and_crop(container, resize_mode, crop_mode)
            .map_err(img::Error::from)?;
        self.compute_transparent_components(self.options)?;
        Ok(())
    }

    #[inline]
    pub fn rotate(&mut self, angle: &types::Rotation) -> Result<(), Error> {
        self.inner.rotate(angle);
        self.compute_transparent_components(self.options)?;
        Ok(())
    }

    #[inline]
    pub fn rotate_to_orientation(&mut self, orientation: types::Orientation) -> Result<(), Error> {
        self.inner.rotate_to_orientation(orientation);
        self.compute_transparent_components(self.options)?;
        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn content_rect(&self) -> Result<&Rect, TransparentComponentsError> {
        self.transparent_components
            .first()
            .ok_or(TransparentComponentsError::Invalid(
                InvalidTransparentComponentsError {
                    required: (Ordering::Greater, 0),
                    components: self.transparent_components.clone(),
                },
            ))
    }

    #[inline]
    #[must_use]
    pub fn content_size(&self) -> Result<Size, Error> {
        let rect = self.content_rect()?;
        let size = rect.size().map_err(|err| error::Arithmetic {
            msg: "failed to compute size of content rect".to_string(),
            source: err.into(),
        })?;
        Ok(size)
    }

    #[inline]
    #[must_use]
    pub fn size_for(
        &self,
        target_content_size: impl Into<types::BoundedSize>,
    ) -> Result<Size, Error> {
        use types::ResizeMode;

        let border_size = self.size();
        let border_content_size = self.content_size()?;
        let target_content_size = target_content_size.into();
        debug!(&border_size);
        debug!(&border_content_size);
        debug!(&target_content_size);

        // scale down if larger than target content size
        let contain_content_size = border_content_size
            .scale_to_bounds(target_content_size, ResizeMode::Contain)
            .map_err(|err| error::Arithmetic {
                msg: "failed to scale border content size".into(),
                source: err.into(),
            })?;
        debug!(&contain_content_size);

        // scale up as little as possible to cover target content size
        let cover_content_size = contain_content_size
            .scale_to_bounds(target_content_size, ResizeMode::Cover)
            .map_err(|err| error::Arithmetic {
                msg: "failed to scale border content size".into(),
                source: err.into(),
            })?;
        debug!(&cover_content_size);

        let border_scale_factor = border_content_size
            .scale_factor(cover_content_size, ResizeMode::Cover)
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute scale factor for border".into(),
                source: err.into(),
            })?;

        debug!(&border_scale_factor);

        let scaled_border_size = self
            .size()
            .scale_by::<_, Round>(border_scale_factor.0)
            .map_err(|err| error::Arithmetic {
                msg: "failed to compute scaled border size".into(),
                source: err.into(),
            })?;

        debug!(&scaled_border_size);
        Ok(scaled_border_size)
    }

    #[inline]
    #[must_use]
    pub fn transparent_components(&self) -> &Vec<Rect> {
        &self.transparent_components
    }
}

fn compute_patch_rect(
    size: Size,
    top_percent: f64,
    bottom_percent: f64,
) -> Result<(Rect, Size), arithmetic::Error> {
    use arithmetic::{ops::CheckedMul, Cast};

    let top_left = Point {
        x: 0,
        y: f64::from(size.height)
            .checked_mul(top_percent)?
            .cast::<i64>()?,
    };
    let bottom_right = Point {
        x: i64::from(size.width),
        y: f64::from(size.height)
            .checked_mul(bottom_percent)?
            .cast::<i64>()?,
    };
    let rect = Rect::from_points(top_left, bottom_right);
    let size = rect.size()?;
    Ok((rect, size))
}

fn overlay_and_fade_patch(
    image: &mut img::Image,
    mut patch: img::Image,
    top_left: Point,
    patch_size: Size,
    fade_height: u32,
) -> Result<(), Error> {
    patch.crop_to_fit(patch_size, types::CropMode::Center);

    #[cfg(feature = "debug")]
    patch
        .clip_alpha(&patch.size().into(), 0, 200)
        .map_err(img::Error::from)?;

    let axis = types::Axis::Y;
    // fade out to top
    let fade_start = Point {
        x: i64::from(patch_size.width),
        y: i64::from(fade_height),
    };
    let fade_end = Point::origin();
    patch.fade_out(fade_start, fade_end, axis);

    // fade out to bottom
    let fade_start = Point::from(patch_size)
        .checked_sub(Point::from(fade_start))
        .map_err(|err| error::Arithmetic {
            msg: "failed to compute fade start point for bottom patch".into(),
            source: err.into(),
        })?;

    let fade_end = Point::from(patch_size);
    patch.fade_out(fade_start, fade_end, axis);
    image.overlay(&patch, top_left);
    Ok(())
}

#[derive(thiserror::Error, PartialEq, Clone, Debug)]
pub enum TransparentComponentsError {
    #[error(transparent)]
    TransparentComponents(#[from] imageops::TransparentComponentsError),

    #[error("invalid transparent components")]
    Invalid(
        #[from]
        #[source]
        InvalidTransparentComponentsError,
    ),
}

#[derive(thiserror::Error, PartialEq, Clone, Debug)]
pub struct InvalidTransparentComponentsError {
    required: (Ordering, usize),
    components: Vec<Rect>,
}

impl std::fmt::Display for InvalidTransparentComponentsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (predicate, required) = self.required;
        let (predicate_string, required) = match predicate {
            Ordering::Less => ("at most", required - 1),
            Ordering::Equal => ("exactly", required),
            Ordering::Greater => ("at least", required + 1),
        };
        write!(
            f,
            "have {} components ({:?}), but {} {} are required",
            self.components.len(),
            self.components,
            predicate_string,
            required
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid transparent components")]
    Invalid(
        #[from]
        #[source]
        InvalidTransparentComponentsError,
    ),

    #[error(transparent)]
    TransparentComponents(#[from] TransparentComponentsError),

    #[error(transparent)]
    Image(#[from] img::Error),

    #[error(transparent)]
    Arithmetic(#[from] error::Arithmetic),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{img::Image, types};
    use anyhow::Result;
    use std::path::{Path, PathBuf};

    fn draw_transparent_components(
        mut border: Image,
        components: &Vec<types::Rect>,
        output: impl AsRef<Path>,
    ) -> Result<()> {
        use imageops::FillMode;

        let red = types::Color::rgba(255, 0, 0, 125);
        for c in components {
            let top_left = Point {
                y: c.top,
                x: c.left,
            };
            let bottom_right = Point {
                y: c.bottom,
                x: c.right,
            };
            border.fill_rect(
                red,
                &Rect::from_points(top_left, bottom_right),
                FillMode::Blend,
            );
        }
        border.save_with_filename(output.as_ref(), None)?;
        Ok(())
    }

    macro_rules! transparent_areas_tests {
        ($($name:ident: $values:expr,)*) => {
            $(
                #[test]
                fn $name() -> Result<()> {
                    let (border_path, expected_components) = $values;
                    let repo: PathBuf = env!("CARGO_MANIFEST_DIR").into();
                    let border_file = repo.join(&border_path);
                    let options = Options::default();
                    let img = Image::open(&border_file)?;
                    let border = Border::from_image(img.clone(), Some(options));
                    let components = match border {
                        Err(TransparentComponentsError::Invalid(InvalidTransparentComponentsError { components, .. })) => Ok(components),
                        Err(err) => Err(err),
                        Ok(border) => {
                            Ok(border.transparent_components().to_vec())
                        }
                    }?;

                    // debug components
                    let output = repo.join(
                        format!("testing/{}.png", stringify!($name)));
                    draw_transparent_components(img, &components, &output)?;
                    println!("components: {:?}", components);

                    assert_eq!(components.len(), expected_components);
                    Ok(())
                }
            )*
        }
    }

    transparent_areas_tests! {
        test_transparent_areas_3_vertical: (
            "samples/borders/border_3_areas_vertical.png", 3),
        test_transparent_areas_3_horizontal: (
            "samples/borders/border_3_areas_horizontal.png", 3),
        test_transparent_areas_1_vertical: (
            "samples/borders/border_1_areas_vertical.png", 1),
        test_transparent_areas_1_horizontal: (
            "samples/borders/border_1_areas_horizontal.png", 1),
    }

    #[test]
    fn test_transparent_areas_3_rotate() -> Result<()> {
        use types::Rotation;

        let repo: PathBuf = env!("CARGO_MANIFEST_DIR").into();
        let border_file = repo.join("samples/borders/border_3_areas_vertical.png");
        let options = Options {
            transparent_component_threshold: 8,
            alpha_threshold: 0.95,
        };
        let img = Image::open(&border_file)?;
        let border = Border::from_image(img, Some(options))?;

        for rotation in &[
            Rotation::Rotate0,
            Rotation::Rotate90,
            Rotation::Rotate180,
            Rotation::Rotate270,
        ] {
            let mut rotated = border.clone();
            rotated.rotate(rotation)?;
            let output = repo.join(format!(
                "testing/border_3_areas_vertical_{:?}.png",
                rotation
            ));
            let components = rotated.transparent_components().clone();
            println!("sorted components: {:?}", &components);
            draw_transparent_components(rotated.inner, &components, &output)?;
            assert_eq!(components.len(), 3);
        }
        Ok(())
    }
}
