use chrono::Utc;
use clap::Parser;
use filmborders::border::{self, Border};
#[cfg(feature = "builtin")]
use filmborders::builtin;
use filmborders::{img, types, Error, ImageBorders};
use std::path::PathBuf;
#[cfg(feature = "builtin")]
use std::str::FromStr;

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "film-borders",
    version = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
    about = "add film borders to an image",
    author = "romnn <contact@romnn.com>",
)]
struct Options {
    #[clap(short = 'i', long = "image")]
    images: Vec<PathBuf>,

    #[clap(short = 'o', long = "output")]
    output: Option<PathBuf>,

    #[clap(short = 'b', long = "border")]
    border: Option<String>,

    #[clap(long = "width")]
    output_width: Option<u32>,

    #[clap(long = "height")]
    output_height: Option<u32>,

    #[clap(long = "max-width")]
    max_output_width: Option<u32>,

    #[clap(long = "max-height")]
    max_output_height: Option<u32>,

    #[clap(long = "margin", aliases = &["margin-factor"])]
    margin: Option<f32>,

    #[clap(long = "scale", aliases = &["scale-factor"])]
    scale_factor: Option<f32>,

    #[clap(long = "crop-top")]
    crop_top: Option<f32>,
    #[clap(long = "crop-right")]
    crop_right: Option<f32>,
    #[clap(long = "crop-bottom")]
    crop_bottom: Option<f32>,
    #[clap(long = "crop-left")]
    crop_left: Option<f32>,

    #[clap(long = "frame-width")]
    frame_width: Option<f32>,

    #[clap(long = "fit", help = "fitting mode")]
    mode: Option<types::FitMode>,

    #[clap(long = "rotate", aliases = &["rotate-image"])]
    image_rotation: Option<types::Rotation>,

    #[clap(long = "rotate-border")]
    border_rotation: Option<types::Rotation>,

    #[clap(long = "background-color", help = "background color in HEX format")]
    background_color: Option<types::Color>,

    #[clap(long = "frame-color", help = "frame color in HEX format")]
    frame_color: Option<types::Color>,

    #[clap(long = "preview", help = "overlay instagram preview visiable area", action = clap::ArgAction::SetTrue)]
    preview: bool,

    #[clap(long = "no-border", action = clap::ArgAction::SetTrue)]
    no_border: bool,

    #[clap(long = "quality", help = "output image quality (1-100)")]
    quality: Option<u8>,

    #[clap(short = 'v', parse(from_occurrences))]
    verbosity: u8,
}

fn main() {
    let options = Options::parse();
    let start = Utc::now().time();
    let images = options
        .images
        .iter()
        .map(|image_path| img::Image::open(image_path).map_err(Error::from))
        .collect::<Result<Vec<img::Image>, Error>>();

    match images.and_then(ImageBorders::new) {
        Ok(mut borders) => {
            let border = if options.no_border {
                None
            } else {
                #[cfg(feature = "builtin")]
                let border = match options.border {
                    None => Ok(border::Kind::default()),
                    Some(border) => builtin::Builtin::from_str(&border)
                        .map(border::Kind::Builtin)
                        .or_else(|_| {
                            Border::open(PathBuf::from(border), None).map(border::Kind::Custom)
                        }),
                };

                #[cfg(not(feature = "builtin"))]
                let border = options
                    .border
                    .ok_or(Error::MissingBorder)
                    .and_then(|border| {
                        Border::open(PathBuf::from(border), None)
                            .map(border::Kind::Custom)
                            .map_err(Into::into)
                    });

                let border = match border {
                    Ok(border) => border,
                    Err(err) => {
                        eprintln!("failed to read border: {:?}", err);
                        return;
                    }
                };
                Some(border)
            };

            let border_options = filmborders::Options {
                output_size: types::BoundedSize {
                    width: options.output_width,
                    height: options.output_height,
                },
                output_size_bounds: types::BoundedSize {
                    width: options.max_output_width,
                    height: options.max_output_height,
                },
                mode: options.mode.unwrap_or_default(),
                crop: Some(types::sides::percent::Sides {
                    top: options.crop_top.unwrap_or(0.0),
                    right: options.crop_right.unwrap_or(0.0),
                    bottom: options.crop_bottom.unwrap_or(0.0),
                    left: options.crop_left.unwrap_or(0.0),
                }),
                scale_factor: options.scale_factor.unwrap_or(1.0),
                margin: options.margin.unwrap_or(0.05),
                frame_width: types::sides::percent::Sides::uniform(
                    options.frame_width.unwrap_or(0.01),
                ),
                image_rotation: options.image_rotation.unwrap_or_default(),
                border_rotation: options.border_rotation.unwrap_or_default(),
                background_color: options.background_color,
                frame_color: options.frame_color.unwrap_or_else(types::Color::black),

                preview: options.preview,
            };
            filmborders::debug!(&border_options);
            match borders
                .add_border(border, &border_options)
                .and_then(|result| match options.output {
                    Some(output) => result
                        .save_with_filename(output, options.quality)
                        .map_err(Error::from),
                    None => result.save(options.quality).map_err(Error::from),
                }) {
                Ok(_) => {
                    println!(
                        "completed in {} msec",
                        (Utc::now().time() - start).num_milliseconds()
                    );
                }
                Err(err) => eprintln!("{}", err),
            };
        }
        Err(err) => eprintln!("{}", err),
    }
}
