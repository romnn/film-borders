mod borders;
mod img;
mod utils;
use clap::Clap;
use std::path::{PathBuf};
use std::time::{Instant};

#[derive(Clap, Debug, Clone)]
struct ApplyOpts {
    #[clap(short = 'i', long = "image")]
    image_path: String,

    #[clap(short = 'o', long = "output")]
    output_path: Option<String>,

    #[clap(short = 'w', long = "width")]
    output_width: Option<u32>,

    #[clap(short = 'h', long = "height")]
    output_height: Option<u32>,

    #[clap(short = 's', long = "scale")]
    scale_factor: Option<f32>,

    #[clap(long = "crop_top")]
    crop_top: Option<u32>,
    #[clap(long = "crop_right")]
    crop_right: Option<u32>,
    #[clap(long = "crop_bottom")]
    crop_bottom: Option<u32>,
    #[clap(long = "crop_left")]
    crop_left: Option<u32>,

    #[clap(short = 'b', long = "border")]
    border_width: Option<u32>,

    #[clap(short = 'r', long = "rotate")]
    rotation: Option<borders::Rotation>,

    #[clap(short = 'p', long = "preview")]
    preview: bool,
}

#[derive(Clap, Debug, Clone)]
#[clap(
    name = "hipster",
    version = "1.0",
    about = "todo",
    author = "romnn <contact@romnn.com>",
    setting = clap::AppSettings::ColoredHelp,
    setting = clap::AppSettings::ArgRequiredElseHelp
)]
enum Hipster {
    #[clap(name = "apply")]
    Apply(ApplyOpts),
}

#[derive(Clap, Debug, Clone)]
#[clap(
    name = "hipster",
    about = "add hipster film borders to images",
    version = "1.0",
    author = "romnn <contact@romnn.com>"
)]
pub struct Opts {
    #[clap(short = 'v', parse(from_occurrences))]
    verbosity: u8,

    #[clap(subcommand)]
    commands: Option<Hipster>,
}

fn main() {
    let opts: Opts = Opts::parse();
    if let Some(subcommand) = opts.commands {
        let start = Instant::now();
        match subcommand {
            Hipster::Apply(cfg) => {
                println!("apply:  {:?}", cfg);
                match img::FilmImage::from_file(PathBuf::from(&cfg.image_path)) {
                    Ok(image) => {
                        let mut b = borders::ImageBorders::new(image);
                        let border_options = borders::ImageBorderOptions {
                            reference_size: None,
                            output_size: Some(borders::Size {
                                width: cfg.output_width.unwrap_or(1080),
                                height: cfg.output_height.unwrap_or(1350),
                            }),
                            crop: Some(borders::Crop {
                                top: cfg.crop_top,
                                right: cfg.crop_right,
                                bottom: cfg.crop_bottom,
                                left: cfg.crop_left,
                            }),
                            scale_factor: Some(cfg.scale_factor.unwrap_or(0.95)),
                            border_width: Some(borders::Sides::uniform(
                                cfg.border_width.unwrap_or(10),
                            )),
                            rotate_angle: Some(cfg.rotation.unwrap_or(borders::Rotation::Rotate0)),
                            preview: cfg.preview,
                        };
                        println!("options:  {:?}", border_options);
                        match b
                            .apply(border_options)
                            .and_then(|result| b.save(result, cfg.output_path))
                        {
                            Ok(_) => println!("done after {:?}", start.elapsed()),
                            Err(err) => println!("{}", err),
                        };
                    }
                    Err(err) => println!("{}", err),
                }
            }
        }
    }
}
