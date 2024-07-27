use anyhow::Result;
use nom::bytes::complete::take;
use nom::number::complete::{be_u32, u8};
use nom::{bytes::complete::tag, IResult};
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct PngChunk<'a> {
    pub length: u32,
    pub chunk_type: &'a str,
    pub data: &'a [u8],
    pub crc: u32,
}

#[derive(Debug)]
pub struct IhdrChunk {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

fn parse_ihdr(input: &[u8]) -> IResult<&[u8], IhdrChunk> {
    let (input, width) = be_u32(input)?;
    let (input, height) = be_u32(input)?;
    let (input, bit_depth) = u8(input)?;
    let (input, color_type) = u8(input)?;
    let (input, compression_method) = u8(input)?;
    let (input, filter_method) = u8(input)?;
    let (input, interlace_method) = u8(input)?;

    Ok((
        input,
        IhdrChunk {
            width,
            height,
            bit_depth,
            color_type,
            compression_method,
            filter_method,
            interlace_method,
        },
    ))
}
fn parse_chunk(input: &[u8]) -> IResult<&[u8], PngChunk> {
    let (input, length) = be_u32(input)?;
    let (input, chunk_type) = take(4usize)(input)?;
    let (input, data) = take(length)(input)?;
    let (input, crc) = be_u32(input)?;

    let chunk_type = std::str::from_utf8(chunk_type).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(
            chunk_type,
            nom::error::ErrorKind::Satisfy,
        ))
    })?;

    Ok((
        input,
        PngChunk {
            length,
            chunk_type,
            data,
            crc,
        },
    ))
}

fn parse_chunks(input: &[u8]) -> IResult<&[u8], Vec<PngChunk>> {
    let mut chunks = Vec::new();
    let mut remaining_input = input;

    while !remaining_input.is_empty() {
        let (rem, chunk) = parse_chunk(remaining_input)?;
        remaining_input = rem;
        chunks.push(chunk);
    }

    Ok((remaining_input, chunks))
}

fn parse_png(input: &[u8]) -> IResult<&[u8], Vec<PngChunk>> {
    const MAGIC_NUMBER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

    let (input, _) = tag(MAGIC_NUMBER)(input)?;

    let (input, chunk) = parse_chunk(input)?;

    let (_, ihdr) = parse_ihdr(chunk.data)?;
    println!("{:?}", ihdr);

    let (input, chunks) = parse_chunks(input)?;

    Ok((input, chunks))
}
fn main() -> Result<()> {
    let mut file = File::open("pixels.png")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let (_, chunks) = parse_png(&buf).unwrap();

    for chunk in chunks {
        println!("{:?}", chunk.chunk_type);
    }

    Ok(())
}
