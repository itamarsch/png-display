use anyhow::Result;
use bitreader::BitReader;
use chunk::PngChunk;
use draw_image::display_pixels;
use ihdr::IhdrChunk;
use nom::{bytes::complete::tag, number::complete::u8, IResult};
use std::fs::File;
use std::io::Read;

use crate::chunk::parse_chunks;
use crate::ihdr::parse_ihdr;
use crate::plte::{parse_palette, Palette, PaletteEntries};

mod chunk;
mod draw_image;
mod ihdr;
mod plte;

fn parse_png(input: &[u8]) -> IResult<&[u8], (IhdrChunk, Vec<PngChunk>)> {
    const MAGIC_NUMBER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

    let (input, _) = tag(MAGIC_NUMBER)(input)?;

    let (input, mut chunks) = parse_chunks(input)?;

    let palette = if let Some(i) = chunks.iter().position(|elem| elem.chunk_type == plte::PLTE) {
        let plte = chunks.remove(i);
        let trns = if let Some(i) = chunks.iter().position(|elem| elem.chunk_type == plte::tRNS) {
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
    let mut file = File::open("pixels.png")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let (_, (ihdr, chunks)) = parse_png(&buf).unwrap();

    let idat = chunks
        .iter()
        .position(|elem| elem.chunk_type == "IDAT")
        .unwrap();

    let mut pixels = vec![];
    let data = chunks[idat..]
        .iter()
        .take_while(|elem| elem.chunk_type == "IDAT")
        .flat_map(|chunk| {
            if ihdr.compression_method == 0 {
                inflate::inflate_bytes_zlib(chunk.data).unwrap()
            } else {
                todo!()
            }
        })
        .collect::<Vec<_>>();

    let mut bitreader = BitReader::new(&data[..]);

    for i in 0..ihdr.height {
        pixels.push(vec![]);
        let filter = bitreader.read_u8(8).unwrap();
        if filter != 0 {
            panic!("Unknown filter: {:?}", filter);
        }
        for _ in 0..ihdr.width {
            match &ihdr.color_type {
                ihdr::ColorType::Grayscale => todo!(),
                ihdr::ColorType::Rgb => todo!(),
                ihdr::ColorType::Palette(Palette {
                    entries: PaletteEntries::RGBA(values),
                }) => {
                    let index = bitreader.read_u8(ihdr.bit_depth)?;
                    let pixel = values[index as usize];
                    pixels[i as usize].push(pixel);
                }

                ihdr::ColorType::Palette(Palette {
                    entries: PaletteEntries::RGB(_),
                }) => todo!(),

                ihdr::ColorType::GrayscaleAlpha => todo!(),
                ihdr::ColorType::Rgba => todo!(),
            }
        }
        let rem_bits = ihdr.width * ihdr.bit_depth as u32 % 8u32;
        _ = bitreader.read_u8(rem_bits as u8);
    }

    display_pixels(pixels, 100);
    Ok(())
}
