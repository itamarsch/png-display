use crate::{chunk::RawChunk, ihdr::IhdrChunk};

use color_print::cprintln;
use text::{CompressedTextChunk, InternationalTextChunk, TextChunk};

use self::background::Background;

pub mod background;
pub mod text;

pub struct AncillaryChunks<'a>(pub Vec<AncillaryChunk<'a>>);

impl AncillaryChunks<'_> {
    pub fn get_background(&self) -> Option<(u8, u8, u8)> {
        self.0
            .iter()
            .filter_map(|s| match s {
                AncillaryChunk::bKGD(b) => Some(b.color),
                _ => None,
            })
            .next()
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AncillaryChunk<'a> {
    bKGD(Background),
    tEXt(TextChunk<'a>),
    zTXt(CompressedTextChunk<'a>),
    iTXt(InternationalTextChunk<'a>),
    Unknown(RawChunk<'a>),
}

impl<'a> AncillaryChunk<'a> {
    fn chunk_type(&self) -> &'a str {
        match &self {
            AncillaryChunk::bKGD(_) => Background::CHUNK_TYPE,
            AncillaryChunk::tEXt(_) => TextChunk::CHUNK_TYPE,
            AncillaryChunk::zTXt(_) => CompressedTextChunk::CHUNK_TYPE,
            AncillaryChunk::iTXt(_) => InternationalTextChunk::CHUNK_TYPE,
            AncillaryChunk::Unknown(c) => c.chunk_type,
        }
    }
}

pub fn parse_ancillary_chunks<'a>(
    chunks: Vec<RawChunk<'a>>,
    ihdr: &IhdrChunk,
) -> Vec<AncillaryChunk<'a>> {
    chunks
        .into_iter()
        .map(|chunk| match chunk.chunk_type {
            TextChunk::CHUNK_TYPE => AncillaryChunk::tEXt(TextChunk::parse(chunk.data).unwrap()),
            CompressedTextChunk::CHUNK_TYPE => {
                AncillaryChunk::zTXt(CompressedTextChunk::parse(chunk.data).unwrap().1)
            }
            InternationalTextChunk::CHUNK_TYPE => {
                AncillaryChunk::iTXt(InternationalTextChunk::parse(chunk.data).unwrap().1)
            }
            Background::CHUNK_TYPE => AncillaryChunk::bKGD(Background::parse(
                chunk.data,
                &ihdr.color_type,
                ihdr.bit_depth,
            )),

            _ => AncillaryChunk::Unknown(chunk),
        })
        .collect()
}
impl<'a> AncillaryChunk<'a> {
    pub fn print(&self) {
        cprintln!("<cyan>Chunk Type: {}</cyan>", self.chunk_type());
        match self {
            AncillaryChunk::tEXt(chunk) => {
                cprintln!("<green>{}</green>", chunk);
            }
            AncillaryChunk::zTXt(chunk) => {
                cprintln!("<green>{}</green>", chunk);
            }
            AncillaryChunk::iTXt(chunk) => {
                cprintln!("<green>{}</green>", chunk);
            }
            AncillaryChunk::bKGD(b) => {
                cprintln!("<green>{:?}</green>", b.color);
            }
            AncillaryChunk::Unknown(_) => {
                cprintln!("<green>Unknown chunk type</green>")
            }
        }
    }
}
