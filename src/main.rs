use anyhow::Context;
use draw_image::display_image;
use std::fs::File;
use std::io::Read;

pub mod ancillary_chunks;
pub mod chunk;
mod color_type;
pub mod draw_image;
pub mod filter_apply;
pub mod ihdr;
pub mod plte;
pub mod png_parser;
pub mod run_n;

fn main() -> anyhow::Result<()> {
    let main_inner = || {
        let filename = std::env::args().nth(1).context("No args passed")?;

        let mut file = File::open(filename)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let png = png_parser::Png::new(&buf)?;

        let pixels = png.get_pixels()?;
        png.print_ancillary();

        let bg = png.other_chunks.get_background();
        let gama = png.other_chunks.get_gama();

        display_image(pixels, 1.0, None, bg, gama)
    };
    let res = main_inner();
    if let Err(err) = &res {
        println!("{}", err.backtrace());
    }
    res
}
