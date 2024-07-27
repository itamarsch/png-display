use anyhow::Result;
use chunk::PngChunk;
use ihdr::IhdrChunk;
use nom::{bytes::complete::tag, IResult};
use std::fs::File;
use std::io::Read;

use crate::chunk::parse_chunks;
use crate::ihdr::parse_ihdr;
use crate::plte::parse_palette;

mod chunk;
mod ihdr;
mod plte;

struct Png {
    ihdr: IhdrChunk,
}

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

    Ok(())
}
