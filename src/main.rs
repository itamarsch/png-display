use anyhow::Result;
use bitreader::BitReader;
use draw_image::display_image;
use filter_apply::decode_scanline;
use std::fs::File;
use std::io::Read;

use crate::plte::{Palette, PaletteEntries};

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

    let (_, png_parser) = png_parser::PngParser::new(&buf).unwrap();
    println!("{}", png_parser.ihdr.bit_depth);

    for c in png_parser.other_chunks {
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

    let mut bitreader = BitReader::new(&png_parser.data[..]);

    let mut pixels =
        vec![vec![(0, 0, 0, 0); png_parser.ihdr.width as usize]; png_parser.ihdr.height as usize];
    let mut prev_scanline = None;
    let bit_depths_per_pixel = match &png_parser.ihdr.color_type {
        ihdr::ColorType::Grayscale => 1,
        ihdr::ColorType::Rgb => 3,
        ihdr::ColorType::Palette(_) => 1,
        ihdr::ColorType::GrayscaleAlpha => 2,
        ihdr::ColorType::Rgba => 4,
    };
    let bits_in_scanline =
        png_parser.ihdr.bit_depth as u32 * bit_depths_per_pixel * png_parser.ihdr.width;
    let scanline_len = 1 + (bits_in_scanline as usize).div_ceil(8);

    let mut scanline = vec![0; scanline_len];
    let mut decoded = vec![0; scanline_len - 1];

    let bpp =
        ((png_parser.ihdr.bit_depth as f32 / 8.0) * bit_depths_per_pixel as f32).round() as usize;
    for i in 0..png_parser.ihdr.height {
        bitreader.read_u8_slice(&mut scanline)?;

        decode_scanline(
            &scanline[..],
            prev_scanline.as_ref().map(|v: &Vec<u8>| &v[..]),
            bpp,
            &mut decoded,
        );

        let mut scanline_reader = BitReader::new(&decoded);

        for j in 0..png_parser.ihdr.width {
            pixels[i as usize][j as usize] = match &png_parser.ihdr.color_type {
                ihdr::ColorType::Grayscale => {
                    if png_parser.ihdr.bit_depth == 8 {
                        let gray_scale = scanline_reader.read_u8(png_parser.ihdr.bit_depth)?;
                        (gray_scale, gray_scale, gray_scale, 255)
                    } else {
                        todo!()
                    }
                }
                ihdr::ColorType::Rgb => todo!(),
                ihdr::ColorType::Palette(Palette {
                    entries: PaletteEntries::RGBA(values),
                }) => {
                    let index = scanline_reader.read_u8(png_parser.ihdr.bit_depth)?;
                    values[index as usize]
                }

                ihdr::ColorType::Palette(Palette {
                    entries: PaletteEntries::RGB(values),
                }) => {
                    let index = scanline_reader.read_u8(png_parser.ihdr.bit_depth)?;
                    let (r, g, b) = values[index as usize];
                    (r, g, b, 255)
                }
                ihdr::ColorType::GrayscaleAlpha => todo!(),
                ihdr::ColorType::Rgba => {
                    if png_parser.ihdr.bit_depth == 8 {
                        let r = scanline_reader.read_u8(8)?;
                        let g = scanline_reader.read_u8(8)?;
                        let b = scanline_reader.read_u8(8)?;
                        let a = scanline_reader.read_u8(8)?;
                        (r, g, b, a)
                    } else {
                        todo!()
                    }
                }
            }
        }

        prev_scanline = Some(decoded.clone());
    }

    display_image(pixels);
    Ok(())
}
