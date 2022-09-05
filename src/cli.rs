use chrono::Utc;
use clap::Parser;
#[cfg(feature = "borders")]
use filmborders::borders;
use filmborders::{img, types, Error, ImageBorders, Options};
use std::path::PathBuf;
#[cfg(feature = "borders")]
use std::str::FromStr;

#[derive(Parser, Debug, Clone)]
struct ApplyOpts {
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
    mode: Option<types::Mode>,

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
}

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "film-borders",
    version = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
    about = "add film borders to an image",
    author = "romnn <contact@romnn.com>",
    arg_required_else_help = true
)]
enum Command {
    #[clap(name = "apply")]
    Apply(ApplyOpts),
}

#[derive(Parser, Debug, Clone)]
#[clap(about = "apply film borders to an image")]
pub struct Opts {
    #[clap(short = 'v', parse(from_occurrences))]
    verbosity: u8,

    #[clap(subcommand)]
    commands: Option<Command>,
}

fn main() {
    let opts: Opts = Opts::parse();
    if let Some(subcommand) = opts.commands {
        let start = Utc::now().time();
        match subcommand {
            Command::Apply(cfg) => {
                filmborders::debug!(&cfg);

                let images = cfg
                    .images
                    .iter()
                    .map(img::Image::open)
                    .collect::<Result<Vec<img::Image>, Error>>();

                match images.and_then(ImageBorders::new) {
                    Ok(mut borders) => {
                        let border = if cfg.no_border {
                            None
                        } else {
                            #[cfg(feature = "borders")]
                            let border = match cfg.border {
                                None => Ok(types::BorderSource::default()),
                                Some(border) => borders::BuiltinBorder::from_str(&border)
                                    .map(types::BorderSource::Builtin)
                                    .or_else(|_| {
                                        types::Border::open(PathBuf::from(border), None)
                                            .map(types::BorderSource::Custom)
                                    }),
                            };

                            #[cfg(not(feature = "borders"))]
                            let border =
                                cfg.border.ok_or(Error::MissingBorder).and_then(|border| {
                                    types::Border::open(PathBuf::from(border), None)
                                        .map(types::BorderSource::Custom)
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

                        let options = Options {
                            output_size: types::OutputSize {
                                width: cfg.output_width,
                                height: cfg.output_height,
                            },
                            output_size_bounds: types::OutputSize {
                                width: cfg.max_output_width,
                                height: cfg.max_output_height,
                            },
                            mode: cfg.mode.unwrap_or_default(),
                            crop: Some(types::SidesPercent {
                                top: cfg.crop_top.unwrap_or(0.0),
                                right: cfg.crop_right.unwrap_or(0.0),
                                bottom: cfg.crop_bottom.unwrap_or(0.0),
                                left: cfg.crop_left.unwrap_or(0.0),
                            }),
                            scale_factor: cfg.scale_factor.unwrap_or(1.0),
                            margin: cfg.margin.unwrap_or(0.05),
                            frame_width: types::SidesPercent::uniform(
                                cfg.frame_width.unwrap_or(0.01),
                            ),
                            image_rotation: cfg.image_rotation.unwrap_or_default(),
                            border_rotation: cfg.border_rotation.unwrap_or_default(),
                            background_color: cfg.background_color,
                            frame_color: cfg.frame_color.unwrap_or_else(types::Color::black),

                            preview: cfg.preview,
                        };
                        filmborders::debug!(&options);
                        match borders
                            .add_border(border, &options)
                            .and_then(|result| result.save(cfg.output, cfg.quality))
                        {
                            Ok(_) => {
                                println!(
                                    "completed in {} msec",
                                    (Utc::now().time() - start).num_milliseconds()
                                )
                            }
                            Err(err) => eprintln!("{}", err),
                        };
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
    }
}
