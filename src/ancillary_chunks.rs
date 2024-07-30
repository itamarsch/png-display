use crate::chunk::RawChunk;

use color_print::cprintln;
use text::{CompressedTextChunk, InternationalTextChunk, TextChunk};

pub mod text;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AncillaryChunk<'a> {
    tEXt(TextChunk<'a>),
    zTXt(CompressedTextChunk<'a>),
    iTXt(InternationalTextChunk<'a>),
    Unknown(RawChunk<'a>),
}

impl<'a> AncillaryChunk<'a> {
    fn chunk_type(&self) -> &'a str {
        match &self {
            AncillaryChunk::tEXt(_) => TextChunk::CHUNK_TYPE,
            AncillaryChunk::zTXt(_) => CompressedTextChunk::CHUNK_TYPE,
            AncillaryChunk::iTXt(_) => InternationalTextChunk::CHUNK_TYPE,
            AncillaryChunk::Unknown(c) => c.chunk_type,
        }
    }
}

pub fn parse_ancillary_chunks(chunks: Vec<RawChunk<'_>>) -> Vec<AncillaryChunk<'_>> {
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
            AncillaryChunk::Unknown(_) => {
                cprintln!("<green>Unknown chunk type</green>")
            }
        }
    }
}
