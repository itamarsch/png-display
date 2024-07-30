use anyhow::Result;
use draw_image::display_image;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

pub mod ancillary_chunks;
pub mod chunk;
mod color_type;
pub mod draw_image;
pub mod filter_apply;
pub mod ihdr;
pub mod plte;
pub mod png_parser;
pub mod run_n;

fn main() -> Result<()> {
    let mut file = File::open(std::env::args().nth(1).unwrap())?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let (_, png) = png_parser::Png::new(&buf).unwrap();

    let pixels = png.get_pixels()?;
    png.print_ancillary();

    let bg = png.other_chunks.get_background();

    display_image(
        pixels,
        500.0 / png.ihdr.width as f32,
        Some(Duration::from_secs_f32(10.0)),
        bg,
    );
    Ok(())
}
