use anyhow::Result;
use bitreader::BitReader;
use chunk::PngChunk;
use draw_image::display_image;
use filter_apply::decode_scanline;
use ihdr::IhdrChunk;
use nom::bytes::streaming::take_until;
use nom::{bytes::complete::tag, IResult};
use std::fs::File;
use std::io::Read;
use text::{parse_iTXt, parse_tEXt};

use crate::chunk::parse_chunks;
use crate::ihdr::parse_ihdr;
use crate::plte::{parse_palette, Palette, PaletteEntries};
use crate::text::parse_zTXt;

mod chunk;
mod draw_image;
mod filter_apply;
mod ihdr;
mod plte;
mod text;

const IDAT: &str = "IDAT";

fn parse_png(input: &[u8]) -> IResult<&[u8], (IhdrChunk, Vec<PngChunk>)> {
    const MAGIC_NUMBER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

    let (input, _) = tag(MAGIC_NUMBER)(input)?;

    let (input, mut chunks) = parse_chunks(input)?;
    println!(
        "{:?}",
        chunks.iter().map(|e| e.chunk_type).collect::<Vec<_>>()
    );

    let palette = if let Some(i) = chunks.iter().position(|elem| elem.chunk_type == plte::PLTE) {
        let plte = chunks.remove(i);
        let trns = if let Some(i) = chunks.iter().position(|elem| elem.chunk_type == plte::TRNS) {
            let trns = chunks.remove(i);
            Some(trns.data)
        } else {
            None
        };

        let (_, palette) = parse_palette(plte.data, trns)?;
        Some(palette)
    } else {
        None
    };

    let ihdr = chunks.remove(0);
    let (_, ihdr) = parse_ihdr(ihdr.data, palette)?;
    println!("Ihdr: {:?}", ihdr);

    Ok((input, (ihdr, chunks)))
}

fn main() -> Result<()> {
    let mut file = File::open(std::env::args().nth(1).unwrap())?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let (_, (ihdr, chunks)) = parse_png(&buf).unwrap();
    let text = chunks
        .iter()
        .filter(|chunk| chunk.chunk_type == "tEXt")
        .collect::<Vec<_>>();
    for chunk in text {
        let (keyword, value) = parse_tEXt(chunk.data)?;
        println!("tEXt. {}: {}", keyword, value);
    }
    let iTXt = chunks
        .iter()
        .filter(|chunk| chunk.chunk_type == "iTXt")
        .collect::<Vec<_>>();

    for chunk in iTXt {
        let (_, itxt) = parse_iTXt(chunk.data).unwrap();
        println!("iTXt. {:?}", itxt);
    }

    let zTXt = chunks
        .iter()
        .filter(|chunk| chunk.chunk_type == "zTXt")
        .collect::<Vec<_>>();

    for chunk in zTXt {
        let (_, (keyword, text)) = parse_zTXt(chunk.data).unwrap();
        println!("zTXt. {:?}, {:?}", keyword, text);
    }

    let idat = chunks
        .iter()
        .position(|elem| elem.chunk_type == "IDAT")
        .unwrap();

    let mut pixels = vec![vec![(0, 0, 0, 0); ihdr.width as usize]; ihdr.height as usize];
    let data = chunks[idat..]
        .iter()
        .take_while(|elem| elem.chunk_type == IDAT)
        .flat_map(|chunk| chunk.data)
        .copied()
        .collect::<Vec<_>>();

    let data = inflate::inflate_bytes_zlib(&data).unwrap();
    let mut bitreader = BitReader::new(&data[..]);

    let mut prev_scanline = None;
    let bit_depths_per_pixel = match &ihdr.color_type {
        ihdr::ColorType::Grayscale => 1,
        ihdr::ColorType::Rgb => 3,
        ihdr::ColorType::Palette(_) => 1,
        ihdr::ColorType::GrayscaleAlpha => 2,
        ihdr::ColorType::Rgba => 4,
    };
    let bits_in_scanline = ihdr.bit_depth as u32 * bit_depths_per_pixel * ihdr.width;
    let scanline_len = 1 + (bits_in_scanline as usize).div_ceil(8);

    let mut scanline = vec![0; scanline_len];
    let mut decoded = vec![0; scanline_len - 1];

    let bpp = ((ihdr.bit_depth as f32 / 8.0) * bit_depths_per_pixel as f32).round() as usize;
    for i in 0..ihdr.height {
        bitreader.read_u8_slice(&mut scanline)?;

        decode_scanline(
            &scanline[..],
            prev_scanline.as_ref().map(|v: &Vec<u8>| &v[..]),
            bpp,
            &mut decoded,
        );

        let mut scanline_reader = BitReader::new(&decoded);

        for j in 0..ihdr.width {
            pixels[i as usize][j as usize] = match &ihdr.color_type {
                ihdr::ColorType::Grayscale => {
                    if ihdr.bit_depth == 8 {
                        let gray_scale = scanline_reader.read_u8(ihdr.bit_depth)?;
                        (gray_scale, gray_scale, gray_scale, 255)
                    } else {
                        todo!()
                    }
                }
                ihdr::ColorType::Rgb => todo!(),
                ihdr::ColorType::Palette(Palette {
                    entries: PaletteEntries::RGBA(values),
                }) => {
                    let index = scanline_reader.read_u8(ihdr.bit_depth)?;
                    values[index as usize]
                }

                ihdr::ColorType::Palette(Palette {
                    entries: PaletteEntries::RGB(values),
                }) => {
                    let index = scanline_reader.read_u8(ihdr.bit_depth)?;
                    let (r, g, b) = values[index as usize];
                    (r, g, b, 255)
                }
                ihdr::ColorType::GrayscaleAlpha => todo!(),
                ihdr::ColorType::Rgba => {
                    if ihdr.bit_depth == 8 {
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
        let rem_bits = ihdr.width * ihdr.bit_depth as u32 % 8u32;
        _ = scanline_reader.read_u8(rem_bits as u8);

        prev_scanline = Some(decoded.clone());
    }

    display_image(pixels);
    Ok(())
}
