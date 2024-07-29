use anyhow::Result;
use draw_image::display_image;
use std::fs::File;
use std::io::Read;

pub mod chunk;
pub mod draw_image;
pub mod filter_apply;
pub mod ihdr;
pub mod plte;
pub mod png_parser;
pub mod text;

fn main() -> Result<()> {
    let mut file = File::open(std::env::args().nth(1).unwrap())?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let (_, png) = png_parser::Png::new(&buf).unwrap();

    for c in &png.other_chunks {
        match c {
            png_parser::Chunk::tEXt(t) => println!("Text\n{}: {}", t.keyword, t.text),
            png_parser::Chunk::zTXt(z) => println!("CompressedText\n{}: {}", z.keyword, z.text),
            png_parser::Chunk::iTXt(i) => println!(
                "InternationalText\n{} {} {}: {}",
                i.keyword, i.language_tag, i.translated_keyword, i.text
            ),
            png_parser::Chunk::Unknown(c) => println!("Unknown\n{}", c.chunk_type),
        }
        println!("---------------")
    }

    let pixels = png.get_pixels()?;

    display_image(pixels);
    Ok(())
}
