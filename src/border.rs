use super::arithmetic::{ops::CheckedSub, Round};
use super::imageops;
use super::img::{self, Image};
use super::types::{self, Point, Rect, Size};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

#[derive(thiserror::Error, Debug)]
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

#[derive(thiserror::Error, Debug)]
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

//     #[error("have {components:?} ({num_components}) but at least {required} are required")]
//     AtLeast {
//         required: usize,
//         components: Vec<super::Rect>,
//         num_components: usize,
//     },

//     #[error("have {components:?} ({num_components}) but exactly {required} is required")]
//     Exact {
//         required: usize,
//         components: Vec<super::Rect>,
//         num_components: usize,
//     },
// }

// #[derive(thiserror::Error, Debug)]
// pub enum InsufficientTransparentComponentsError {
//     #[error("have {components:?} ({num_components}) but at least {required} are required")]
//     AtLeast {
//         required: usize,
//         components: Vec<super::Rect>,
//         num_components: usize,
//     },

//     #[error("have {components:?} ({num_components}) but exactly {required} is required")]
//     Exact {
//         required: usize,
//         components: Vec<super::Rect>,
//         num_components: usize,
//     },
// }

#[derive(thiserror::Error, Debug)]
pub enum CustomBorderError {
    #[error("invalid transparent components")]
    Invalid(
        #[from]
        #[source]
        InvalidTransparentComponentsError,
    ),

    #[error(transparent)]
    Border(#[from] Error),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // #[error("insufficient transparent components")]
    // InsufficientTransparentComponents(
    //     #[from]
    //     #[source]
    //     InsufficientTransparentComponentsError,
    // ),
    // #[error(transparent)]
    // TransparentComponents(#[from] TransparentComponentsError),
    #[error(transparent)]
    TransparentComponents(#[from] TransparentComponentsError),

    #[error(transparent)]
    Image(#[from] img::Error),
}

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
    ) -> Result<Self, CustomBorderError> {
        use super::arithmetic::{
            ops::{CheckedAdd, CheckedDiv, CheckedMul},
            Cast,
        };

        let components = border.transparent_components().to_vec();
        if components.len() != 1 {
            return Err(CustomBorderError::Invalid(
                InvalidTransparentComponentsError {
                    required: (Ordering::Equal, 1),
                    components,
                },
            ));
        }
        // by default, use the longer dimension to stich
        let stich_direction = stich_direction.unwrap_or(types::Orientation::Portrait);
        border.rotate_to_orientation(stich_direction)?;
        let target_content_size = content_size.rotate_to_orientation(stich_direction);
        let border_size = border.size_for(types::BoundedSize {
            width: Some(target_content_size.width),
            height: None,
        });
        crate::debug!(&border_size);
        border
            .resize_and_crop(border_size, types::ResizeMode::Cover)
            .unwrap();
        crate::debug!(&border.content_size());

        // border is portrait now, we stich vertically
        // todo: find optimal overlay patches somehow
        let top_patch_rect = {
            let top_left = Point { x: 0, y: 0 };
            let bottom_right = Point {
                x: i64::from(border_size.width),
                y: f64::from(border_size.height)
                    .checked_mul(0.25)
                    .unwrap()
                    .cast::<i64>()
                    .unwrap(),
            };
            Rect::from_points(top_left, bottom_right)
        };
        let top_patch_size = top_patch_rect.size().unwrap();

        let bottom_patch_rect = {
            let top_left = Point {
                x: 0,
                y: f64::from(border_size.height)
                    .checked_mul(0.75)
                    .unwrap()
                    .cast::<i64>()
                    .unwrap(),
            };
            let bottom_right = Point {
                x: i64::from(border_size.width),
                y: i64::from(border_size.height),
            };
            Rect::from_points(top_left, bottom_right)
        };
        let bottom_patch_size = bottom_patch_rect.size().unwrap();

        let overlay_patch_rect = {
            let top_left = Point {
                x: 0,
                y: f64::from(border_size.height)
                    .checked_mul(0.3)
                    .unwrap()
                    .cast::<i64>()
                    .unwrap(),
            };
            let bottom_right = Point {
                x: i64::from(border_size.width),
                y: f64::from(border_size.height)
                    .checked_mul(0.7)
                    .unwrap()
                    .cast::<i64>()
                    .unwrap(),
            };
            Rect::from_points(top_left, bottom_right)
        };
        let overlay_patch_size = overlay_patch_rect.size().unwrap();

        // create buffer for the new border
        let border_padding = border_size.checked_sub(border.content_size()).unwrap();
        let new_border_size = target_content_size.checked_add(border_padding).unwrap();
        let mut new_border = Image::with_size(new_border_size);
        crate::debug!(&new_border.size());

        #[cfg(debug_assertions)]
        {
            use imageops::FillMode;
            let green = types::Color::rgba(0, 100, 0, 255);
            let clear = types::Color::clear();
            new_border.fill(green, FillMode::Set);
            new_border.fill_rect(clear, border.content_rect(), FillMode::Set);
        }

        // draw top patch
        let mut border_top: Image = (*border).clone();
        border_top.crop(&top_patch_rect);
        new_border.overlay(&border_top, Point::origin());

        // draw bottom patch
        let mut border_bottom = border.inner.clone();
        border_bottom.crop(&bottom_patch_rect);

        let bottom_patch_top_left: Point = Point::from(new_border_size)
            .checked_sub(bottom_patch_size.into())
            .unwrap();
        new_border.overlay(&border_bottom, bottom_patch_top_left);

        // draw patches in between
        let fill_height = i64::from(new_border_size.height)
            .checked_sub(i64::from(bottom_patch_size.height))
            .unwrap()
            .checked_sub(i64::from(top_patch_size.height))
            .unwrap()
            .max(1)
            .cast::<u32>()
            .unwrap();
        crate::debug!(&fill_height);

        let fade_height = f64::from(overlay_patch_size.height)
            .checked_mul(0.2)
            .unwrap()
            .ceil()
            .cast::<u32>()
            .unwrap();
        let fade_size = Size {
            width: overlay_patch_size.width,
            height: fade_height,
        };
        let patch_safe_height = overlay_patch_size.height - 2 * fade_size.height;

        let num_patches = f64::from(fill_height)
            .checked_div(f64::from(patch_safe_height))
            .unwrap()
            .ceil()
            .cast::<u32>()
            .unwrap();
        assert!(num_patches > 0);
        crate::debug!(&num_patches);

        let patch_safe_height = f64::from(fill_height)
            .checked_div(f64::from(num_patches))
            .unwrap()
            .ceil()
            .cast::<u32>()
            .unwrap();
        let patch_height = patch_safe_height
            .checked_add(fade_size.height.checked_mul(2).unwrap())
            .unwrap();
        let patch_size = Size {
            width: overlay_patch_size.width,
            height: patch_height,
        };

        for i in 0..num_patches {
            let mut border_overlay_patch = border.inner.clone();
            border_overlay_patch.crop_to_fit(patch_size, types::CropMode::Center);
            let axis = types::Axis::Y;
            border_overlay_patch.fade_out(fade_size, Point::origin(), axis);
            border_overlay_patch.fade_out(
                Point::from(patch_size)
                    .checked_sub(Point::from(fade_size))
                    .unwrap(),
                patch_size,
                axis,
            );
            let patch_offset_y = i64::from(i)
                .checked_mul(
                    i64::from(patch_safe_height)
                        .checked_sub(i64::from(fade_height))
                        .unwrap(),
                )
                .unwrap();
            let patch_top_left = top_patch_rect
                .bottom_left()
                .checked_add(Point {
                    x: 0,
                    y: patch_offset_y,
                })
                .unwrap();
            new_border.overlay(&border_overlay_patch, patch_top_left);
        }

        let mut new_border = Self::from_image(new_border, border.options).unwrap();

        // match orientation to target content size
        new_border
            .rotate_to_orientation(content_size.orientation())
            .unwrap();
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
            .map_err(img::Error::from);
        self.compute_transparent_components(self.options).unwrap();
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
    pub fn content_rect(&self) -> &Rect {
        self.transparent_components.first().unwrap()
    }

    #[inline]
    #[must_use]
    pub fn content_size(&self) -> Size {
        self.content_rect().size().unwrap()
    }

    #[inline]
    #[must_use]
    pub fn size_for<S: Into<types::BoundedSize>>(&self, target_content_size: S) -> Size {
        let content_size = self.content_size();
        let target_content_size = target_content_size.into();
        crate::debug!(&content_size);
        crate::debug!(&target_content_size);

        // scale down if larget than target content size
        let new_content_size = content_size
            .scale_to_bounds(target_content_size, types::ResizeMode::Contain)
            .unwrap();
        crate::debug!(&new_content_size);

        // scale up as little as possible to cover target content size
        let new_content_size = new_content_size
            .scale_to_bounds(target_content_size, types::ResizeMode::Cover)
            .unwrap();

        crate::debug!(&new_content_size);
        let scale_factor = content_size
            .scale_factor(new_content_size, types::ResizeMode::Cover)
            .unwrap();
        self.size().scale_by::<_, Round>(scale_factor.0).unwrap()
    }

    #[inline]
    #[must_use]
    pub fn transparent_components(&self) -> &Vec<Rect> {
        &self.transparent_components
    }
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
            // let size: Size = bottom_right
            //     .checked_sub(top_left)
            //     .unwrap()
            //     .try_into()
            //     .unwrap();
            border.fill_rect(
                red,
                &Rect::from_points(top_left, bottom_right),
                FillMode::Blend,
            );
            // border.fill_rect(red, top_left, size, FillMode::Blend);
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
