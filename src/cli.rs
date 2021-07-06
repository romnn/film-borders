mod borders;
mod fs;
mod img;
mod utils;
use clap::Clap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Clap, Debug, Clone)]
struct ApplyOpts {
    #[clap(short = 'i', long = "image")]
    image_path: String,

    #[clap(short = 'o', long = "output")]
    output_path: Option<String>,
    // #[clap(short = 'p', long = "port", default_value = "3000")]
    // port: u16,

    // #[clap(short = 'n', long = "pages")]
    // max_pages: Option<u32>,
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
    about = "TODO",
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
                // match img::ImageBorders::new(PathBuf::from(&cfg.image_path)) {
                match img::FilmImage::from_file(PathBuf::from(&cfg.image_path)) {
                    Ok(image) => {
                        let b = borders::ImageBorders::new(image);
                        let border_options = borders::ImageBorderOptions {
                            output_size: None,
                            crop: None,
                            border_width: Some(borders::Sides {
                                top: 10,
                                ..borders::Sides::default()
                            }),
                            padding: Some(borders::Sides {
                                top: 10,
                                ..borders::Sides::default()
                            }),
                            rotate_angle: Some(90),
                        };
                        b.apply(border_options);
                        b.save(cfg.output_path, Some(80));
                        println!("done after {:?}", start.elapsed());
                    }
                    Err(err) => println!("{}", err),
                }
            }
        }
    }
}
