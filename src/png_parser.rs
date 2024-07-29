use crate::chunk::RawChunk;
use crate::ihdr::IhdrChunk;
use nom::{bytes::complete::tag, IResult};

use crate::chunk::parse_chunks;
use crate::ihdr::parse_ihdr;
use crate::plte::{self, parse_palette, Palette};

use crate::text::{CompressedTextChunk, InternationalTextChunk, TextChunk};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Chunk<'a> {
    tEXt(TextChunk<'a>),
    zTXt(CompressedTextChunk<'a>),
    iTXt(InternationalTextChunk<'a>),
    Unknown(RawChunk<'a>),
}

pub struct PngParser<'a> {
    pub ihdr: IhdrChunk,
    pub data: Vec<u8>,
    pub other_chunks: Vec<Chunk<'a>>,
}

fn take_palette_chunk(chunks: &mut Vec<RawChunk>) -> Option<Palette> {
    if let Some(i) = chunks.iter().position(|elem| elem.chunk_type == plte::PLTE) {
        let plte = chunks.remove(i);
        let trns = if let Some(i) = chunks.iter().position(|elem| elem.chunk_type == plte::TRNS) {
            let trns = chunks.remove(i);
            Some(trns.data)
        } else {
            None
        };

        let (_, palette) = parse_palette(plte.data, trns).unwrap();
        Some(palette)
    } else {
        None
    }
}

const IDAT: &str = "IDAT";
fn take_idta_chunks(chunks: &mut Vec<RawChunk>) -> Vec<u8> {
    let first_idat_index = chunks
        .iter()
        .position(|elem| elem.chunk_type == IDAT)
        .unwrap();

    let idat_indexes = chunks[first_idat_index..]
        .iter()
        .enumerate()
        .take_while(|(_, elem)| elem.chunk_type == IDAT)
        .map(|(i, _)| i + first_idat_index)
        .collect::<Vec<_>>();

    let data: Vec<u8> = idat_indexes
        .iter()
        .flat_map(|&index| chunks[index].data)
        .copied()
        .collect();

    // Remove IDAT chunks from the vector
    for &index in idat_indexes.iter().rev() {
        chunks.remove(index);
    }

    inflate::inflate_bytes_zlib(&data).unwrap()
}

fn parse_non_required_chunks<'a>(chunks: Vec<RawChunk<'a>>) -> Vec<Chunk<'a>> {
    chunks
        .into_iter()
        .map(|chunk| match chunk.chunk_type {
            TextChunk::CHUNK_TYPE => Chunk::tEXt(TextChunk::parse(chunk.data).unwrap()),
            CompressedTextChunk::CHUNK_TYPE => {
                Chunk::zTXt(CompressedTextChunk::parse(chunk.data).unwrap().1)
            }
            InternationalTextChunk::CHUNK_TYPE => {
                Chunk::iTXt(InternationalTextChunk::parse(chunk.data).unwrap().1)
            }
            _ => Chunk::Unknown(chunk),
        })
        .collect()
}

impl<'a> PngParser<'a> {
    pub fn new(input: &'a [u8]) -> IResult<&'a [u8], Self> {
        const MAGIC_NUMBER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

        let (input, _) = tag(MAGIC_NUMBER)(input)?;

        let (input, mut chunks) = parse_chunks(input)?;

        let palette = take_palette_chunk(&mut chunks);

        let ihdr = chunks.remove(0);
        let (_, ihdr) = parse_ihdr(ihdr.data, palette)?;

        let data = take_idta_chunks(&mut chunks);

        let non_requied_chunks = parse_non_required_chunks(chunks);
        Ok((
            input,
            Self {
                ihdr,
                data,
                other_chunks: non_requied_chunks,
            },
        ))
    }
}
