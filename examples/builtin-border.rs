#[cfg(not(feature = "builtin"))]
fn main() {
    panic!(r#"Feature "builtin" must be enabled for this example"#);
}

#[cfg(feature = "builtin")]
fn main() -> anyhow::Result<()> {
    use filmborders::{builtin, types, Image, ImageBorders, Options};
    use std::path::PathBuf;

    let root = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let image1 = Image::open(root.join("samples/lowres.jpg"))?;
    let image2 = Image::open(root.join("samples/lowres2.jpg"))?;

    let mut borders = ImageBorders::new([image1, image2])?;
    let border = builtin::Builtin::Border120_1;

    let options = Options {
        output_size: types::BoundedSize {
            width: Some(1000),
            height: Some(1000),
        },
        mode: types::FitMode::Image,
        margin: 0.05,
        frame_width: types::SidesPercent::uniform(0.01),
        background_color: Some(types::Color::rgba(200, 255, 255, 255)),
        frame_color: types::Color::black(),
        ..Default::default()
    };

    let result = borders.render(border, &options)?;

    let example_file_name = PathBuf::from(file!())
        .file_name()
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow::anyhow!("missing example filename"))?;

    let output_path = root
        .join("examples")
        .join(example_file_name)
        .with_extension("jpg");

    result.save_with_filename(&output_path, None)?;
    println!("saved to {:?}", &output_path);

    Ok(())
}
