use chrono::Utc;
use clap::Parser;
use filmborders::{BorderOptions, Crop, ImageBorders, OutputSize, Rotation, Sides};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
struct ApplyOpts {
    #[clap(short = 'i', long = "image")]
    image: PathBuf,

    #[clap(short = 'o', long = "output")]
    output: Option<PathBuf>,

    #[clap(long = "width")]
    output_width: Option<u32>,

    #[clap(long = "height")]
    output_height: Option<u32>,

    #[clap(long = "scale")]
    scale_factor: Option<f32>,

    #[clap(long = "crop-top")]
    crop_top: Option<u32>,
    #[clap(long = "crop-right")]
    crop_right: Option<u32>,
    #[clap(long = "crop-bottom")]
    crop_bottom: Option<u32>,
    #[clap(long = "crop-left")]
    crop_left: Option<u32>,

    #[clap(long = "border-width")]
    border_width: Option<u32>,

    #[clap(long = "rotate")]
    rotation: Option<Rotation>,

    #[clap(long = "preview", help = "overlay instagram preview visiable area", action = clap::ArgAction::SetTrue)]
    preview: bool,

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
                match ImageBorders::open(&cfg.image) {
                    Ok(mut borders) => {
                        let options = BorderOptions {
                            output_size: Some(OutputSize {
                                width: cfg.output_width,
                                height: cfg.output_height,
                            }),
                            crop: Some(Crop {
                                top: cfg.crop_top,
                                right: cfg.crop_right,
                                bottom: cfg.crop_bottom,
                                left: cfg.crop_left,
                            }),
                            scale_factor: Some(cfg.scale_factor.unwrap_or(0.95)),
                            border_width: Some(Sides::uniform(cfg.border_width.unwrap_or(10))),
                            rotate_angle: Some(cfg.rotation.unwrap_or(Rotation::Rotate0)),
                            preview: cfg.preview,
                        };
                        filmborders::debug!(&options);
                        match borders
                            .apply(options)
                            .and_then(|result| result.save(cfg.output, cfg.quality))
                        {
                            Ok(_) => println!("completed in {:?}", Utc::now().time() - start),
                            Err(err) => eprintln!("{}", err),
                        };
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
    }
}
