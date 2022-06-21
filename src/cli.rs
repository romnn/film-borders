use filmborders::ImageBorders;
use filmborders::Image;
use filmborders::options;
use filmborders::types;
use chrono::Utc;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
struct ApplyOpts {
    #[clap(short = 'i', long = "image")]
    image_path: PathBuf,

    #[clap(short = 'o', long = "output")]
    output_path: Option<PathBuf>,

    #[clap(long = "width")]
    output_width: Option<u32>,

    #[clap(long = "height")]
    output_height: Option<u32>,

    #[clap(long = "scale")]
    scale_factor: Option<f32>,

    #[clap(long = "crop_top")]
    crop_top: Option<u32>,
    #[clap(long = "crop_right")]
    crop_right: Option<u32>,
    #[clap(long = "crop_bottom")]
    crop_bottom: Option<u32>,
    #[clap(long = "crop_left")]
    crop_left: Option<u32>,

    #[clap(long = "border")]
    border_width: Option<u32>,

    #[clap(long = "rotate")]
    rotation: Option<types::Rotation>,

    #[clap(long = "preview")]
    preview: bool,
}

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "film-borders",
    version = "1.0",
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
                // println!("apply:  {:?}", cfg);
                match Image::from_file(&cfg.image_path) {
                    Ok(image) => {
                        let mut b = ImageBorders::new(image);
                        let border_options = options::BorderOptions {
                            output_size: Some(types::OutputSize {
                                width: cfg.output_width,
                                height: cfg.output_height,
                            }),
                            crop: Some(types::Crop {
                                top: cfg.crop_top,
                                right: cfg.crop_right,
                                bottom: cfg.crop_bottom,
                                left: cfg.crop_left,
                            }),
                            scale_factor: Some(cfg.scale_factor.unwrap_or(0.95)),
                            border_width: Some(types::Sides::uniform(
                                cfg.border_width.unwrap_or(10),
                            )),
                            rotate_angle: Some(cfg.rotation.unwrap_or(types::Rotation::Rotate0)),
                            preview: cfg.preview,
                        };
                        // println!("options:  {:?}", border_options);
                        match b.apply(border_options).and_then(|result| {
                            b.save(result, cfg.output_path.map(|p| p.as_path().to_owned()))
                        }) {
                            Ok(_) => println!("done after {:?}", Utc::now().time() - start),
                            Err(err) => println!("{}", err),
                        };
                    }
                    Err(err) => println!("{}", err),
                }
            }
        }
    }
}
