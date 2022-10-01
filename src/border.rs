use super::arithmetic::{ops::CheckedSub, Round};
use super::img;
use super::types::{Point, Size};
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("border must contain at least one transparent area, found {0:?}")]
    BadTransparency(Vec<super::Rect>),

    #[error("invalid border: {0}")]
    Invalid(String),
}

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub transparent_component_threshold: i64,
    pub alpha_threshold: f32,
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
    pub fn into_border(self) -> Result<Border, super::Error> {
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
    inner: img::Image,
    options: Option<Options>,
    transparent_components: Vec<super::Rect>,
}

impl Border {
    #[inline]
    pub fn from_reader<R: std::io::BufRead + std::io::Seek>(
        reader: R,
        options: Option<Options>,
    ) -> Result<Self, super::Error> {
        Self::from_image(img::Image::from_reader(reader)?, options)
    }

    #[inline]
    pub fn open<P: AsRef<Path>>(path: P, options: Option<Options>) -> Result<Self, super::Error> {
        Self::from_image(img::Image::open(path)?, options)
    }

    #[inline]
    pub fn new(
        mut border: Self,
        content_size: Size,
        stich_direction: Option<super::Orientation>,
    ) -> Result<Self, super::Error> {
        use super::arithmetic::{
            ops::{CheckedAdd, CheckedDiv, CheckedMul},
            Cast,
        };
        use super::{BoundedSize, Orientation, Rect, ResizeMode};

        let comps = border.transparent_components().len();
        if comps != 1 {
            return Err(Error::Invalid(format!(
                "border must only have one transparent area, found {}",
                comps
            ))
            .into());
        }
        // by default, use the longer dimension to stich
        let stich_direction = stich_direction.unwrap_or(Orientation::Portrait);
        border.rotate_to_orientation(stich_direction)?;
        let target_content_size = content_size.rotate_to_orientation(stich_direction);
        let border_size = border.size_for(BoundedSize {
            width: Some(target_content_size.width),
            height: None,
        });
        crate::debug!(&border_size);
        border.resize_to_fit(border_size, ResizeMode::Cover)?;
        crate::debug!(&border.content_size());

        // border is portrait now, we stich vertically
        // todo: find optimal overlay patches somehow
        let top_patch = Rect {
            top: 0,
            bottom: f64::from(border_size.height)
                .checked_mul(0.25)
                .unwrap()
                .cast::<i64>()
                .unwrap(),
            left: 0,
            right: i64::from(border_size.width),
        };
        let top_patch_size = top_patch.size().unwrap();

        let bottom_patch = Rect {
            top: f64::from(border_size.height)
                .checked_mul(0.75)
                .unwrap()
                .cast::<i64>()
                .unwrap(),
            bottom: i64::from(border_size.height),
            left: 0,
            right: i64::from(border_size.width),
        };
        let bottom_patch_size = bottom_patch.size().unwrap();

        let overlay_patch = Rect {
            top: f64::from(border_size.height)
                .checked_mul(0.3)
                .unwrap()
                .cast::<i64>()
                .unwrap(),
            bottom: f64::from(border_size.height)
                .checked_mul(0.7)
                .unwrap()
                .cast::<i64>()
                .unwrap(),
            left: 0,
            right: i64::from(border_size.width),
        };
        let overlay_patch_size = overlay_patch.size().unwrap();

        // create buffer for the new border
        let border_padding = border_size.checked_sub(border.content_size()).unwrap();
        let new_border_size = target_content_size.checked_add(border_padding).unwrap();
        let mut new_border = img::Image::with_size(new_border_size);
        crate::debug!(&new_border.size());

        #[cfg(debug_assertions)]
        {
            use super::imageops::FillMode;
            new_border.fill(super::Color::rgba(0, 100, 0, 255), FillMode::Set);
            new_border.fill_rect(
                super::Color::clear(),
                border.content_rect().top_left(),
                target_content_size,
                FillMode::Set,
            );
        }

        // draw top patch
        let mut border_top = border.inner.clone();
        border_top.crop(top_patch.top_left(), top_patch.bottom_right());
        new_border.overlay(border_top.as_ref(), Point::origin());

        // draw bottom patch
        let mut border_bottom = border.inner.clone();
        border_bottom.crop(bottom_patch.top_left(), bottom_patch.bottom_right());
        // let bottom_patch_size = bottom_patch.size().unwrap();
        // let top_patch_size = top_patch.size().unwrap();

        let bottom_patch_top_left: Point = Point::from(new_border_size)
            .checked_sub(bottom_patch_size.into())
            .unwrap();
        new_border.overlay(border_bottom.as_ref(), bottom_patch_top_left);

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
            border_overlay_patch.crop_to_fit(patch_size, super::CropMode::Center);
            let axis = super::Axis::Y;
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
            let patch_top_left = top_patch
                .bottom_left()
                .checked_add(Point {
                    x: 0,
                    y: patch_offset_y,
                })
                .unwrap();
            new_border.overlay(border_overlay_patch.as_ref(), patch_top_left);
        }

        let mut new_border = Self::from_image(new_border, border.options)?;

        // match orientation to target content size
        new_border.rotate_to_orientation(content_size.orientation())?;
        Ok(new_border)
    }

    #[inline]
    fn compute_transparent_components(&mut self, options: Option<Options>) -> Result<(), Error> {
        use super::imageops::find_transparent_components;

        let options = options.unwrap_or_default();
        self.transparent_components = find_transparent_components(
            &self.inner,
            options.alpha_threshold,
            options.transparent_component_threshold,
        );

        if self.transparent_components.is_empty() {
            return Err(Error::BadTransparency(self.transparent_components.clone()));
        }
        self.transparent_components
            .sort_by_key(|b| std::cmp::Reverse(b.pixel_count().unwrap()));
        // .sort_by(|a, b| b.pixel_count().unwrap().cmp(&a.pixel_count().unwrap()));
        Ok(())
    }

    #[inline]
    pub fn from_image(inner: img::Image, options: Option<Options>) -> Result<Self, super::Error> {
        let mut border = Self {
            inner,
            options,
            transparent_components: Vec::new(),
        };
        border.compute_transparent_components(options)?;
        Ok(border)
    }

    #[inline]
    pub fn resize_to_fit(
        &mut self,
        container: Size,
        resize_mode: super::ResizeMode,
    ) -> Result<(), super::Error> {
        self.inner
            .resize_to_fit(container, resize_mode, super::CropMode::Center);
        self.compute_transparent_components(self.options)?;
        Ok(())
    }

    #[inline]
    pub fn rotate(&mut self, angle: &super::Rotation) -> Result<(), super::Error> {
        self.inner.rotate(angle);
        self.compute_transparent_components(self.options)?;
        Ok(())
    }

    #[inline]
    pub fn rotate_to_orientation(
        &mut self,
        orientation: super::Orientation,
    ) -> Result<(), super::Error> {
        if self.inner.orientation() != orientation {
            self.rotate(&super::Rotation::Rotate90)?;
        }
        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn content_rect(&self) -> &super::Rect {
        self.transparent_components.first().unwrap()
    }

    #[inline]
    #[must_use]
    pub fn content_size(&self) -> Size {
        self.content_rect().size().unwrap()
    }

    #[inline]
    #[must_use]
    pub fn size_for<S: Into<super::BoundedSize>>(&self, target_content_size: S) -> Size {
        use super::ResizeMode;

        let content_size = self.content_size();
        let target_content_size = target_content_size.into();
        crate::debug!(&content_size);
        crate::debug!(&target_content_size);

        // scale down if larget than target content size
        let new_content_size =
            content_size.scale_to_bounds(target_content_size, ResizeMode::Contain);
        crate::debug!(&new_content_size);

        // scale up as little as possible to cover target content size
        let new_content_size =
            new_content_size.scale_to_bounds(target_content_size, ResizeMode::Cover);

        crate::debug!(&new_content_size);
        let scale_factor = content_size.scale_factor(new_content_size, ResizeMode::Cover);
        self.size().scale_by::<_, Round>(scale_factor.0).unwrap()
    }

    #[inline]
    #[must_use]
    pub fn transparent_components(&self) -> &Vec<super::Rect> {
        &self.transparent_components
    }

    #[inline]
    #[must_use]
    pub fn size(&self) -> Size {
        self.inner.size()
    }

    #[inline]
    #[must_use]
    pub fn orientation(&self) -> super::Orientation {
        self.size().orientation()
    }
}

impl AsRef<image::RgbaImage> for Border {
    fn as_ref(&self) -> &image::RgbaImage {
        self.inner.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{img, types};
    use anyhow::Result;
    use std::path::{Path, PathBuf};

    fn draw_transparent_components(
        mut border: img::Image,
        components: &Vec<types::Rect>,
        output: impl AsRef<Path>,
    ) -> Result<()> {
        use crate::imageops::FillMode;

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
            let size: Size = bottom_right
                .checked_sub(top_left)
                .unwrap()
                .try_into()
                .unwrap();
            border.fill_rect(red, top_left, size, FillMode::Blend);
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
                    let img = img::Image::open(&border_file)?;
                    let border = Border::from_image(img.clone(), Some(options));
                    let components = match border {
                        Err(crate::Error::Border(Error::BadTransparency(c))) => Ok(c),
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
        let img = img::Image::open(&border_file)?;
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
