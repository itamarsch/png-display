use crate::{chunk::RawChunk, ihdr::IhdrChunk};

use anyhow::Result;
use color_print::cprintln;
use text::{CompressedTextChunk, InternationalTextChunk, TextChunk};

use self::{background::Background, gama::Gama, phys::PhysicalUnits, time::Time};

pub mod background;
pub mod gama;
pub mod phys;
pub mod text;
pub mod time;

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

    pub fn get_gama(&self) -> Option<Gama> {
        self.0
            .iter()
            .filter_map(|s| match s {
                AncillaryChunk::gAMA(b) => Some(*b),
                _ => None,
            })
            .next()
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AncillaryChunk<'a> {
    tIME(Time),
    gAMA(Gama),
    bKGD(Background),
    tEXt(TextChunk<'a>),
    zTXt(CompressedTextChunk<'a>),
    iTXt(InternationalTextChunk<'a>),
    pHYs(PhysicalUnits),
    Unknown(RawChunk<'a>),
}

impl<'a> AncillaryChunk<'a> {
    pub fn chunk_type(&self) -> &'a str {
        match &self {
            AncillaryChunk::pHYs(_) => PhysicalUnits::CHUNK_TYPE,
            AncillaryChunk::bKGD(_) => Background::CHUNK_TYPE,
            AncillaryChunk::tEXt(_) => TextChunk::CHUNK_TYPE,
            AncillaryChunk::zTXt(_) => CompressedTextChunk::CHUNK_TYPE,
            AncillaryChunk::iTXt(_) => InternationalTextChunk::CHUNK_TYPE,
            AncillaryChunk::tIME(_) => Time::CHUNK_TYPE,
            AncillaryChunk::gAMA(_) => Gama::CHUNK_TYPE,
            AncillaryChunk::Unknown(c) => c.chunk_type,
        }
    }
}

pub fn parse_ancillary_chunks<'a>(
    chunks: Vec<RawChunk<'a>>,
    ihdr: &IhdrChunk,
) -> anyhow::Result<Vec<AncillaryChunk<'a>>> {
    chunks
        .into_iter()
        .map(|chunk| -> anyhow::Result<_> {
            match chunk.chunk_type {
                TextChunk::CHUNK_TYPE => Ok(AncillaryChunk::tEXt(TextChunk::parse(chunk.data)?)),
                CompressedTextChunk::CHUNK_TYPE => Ok(AncillaryChunk::zTXt(
                    CompressedTextChunk::parse(chunk.data)?,
                )),
                InternationalTextChunk::CHUNK_TYPE => Ok(AncillaryChunk::iTXt(
                    InternationalTextChunk::parse(chunk.data)?,
                )),
                Background::CHUNK_TYPE => Ok(AncillaryChunk::bKGD(Background::parse(
                    chunk.data,
                    &ihdr.color_type,
                    ihdr.bit_depth,
                )?)),
                Time::CHUNK_TYPE => Ok(AncillaryChunk::tIME(Time::parse(chunk.data)?)),
                PhysicalUnits::CHUNK_TYPE => Ok(AncillaryChunk::pHYs(PhysicalUnits::parse(
                    chunk.data,
                    ihdr.width,
                    ihdr.height,
                )?)),
                Gama::CHUNK_TYPE => Ok(AncillaryChunk::gAMA(Gama::parse(chunk.data)?)),

                _ => Ok(AncillaryChunk::Unknown(chunk)),
            }
        })
        .collect::<Result<Vec<_>, _>>()
}
impl<'a> AncillaryChunk<'a> {
    pub fn print(&self) {
        cprintln!("<cyan>Chunk Type: {}</cyan>", self.chunk_type());
        match self {
            AncillaryChunk::gAMA(chunk) => {
                cprintln!("<green>{}</green>", chunk);
            }

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
            AncillaryChunk::tIME(chunk) => {
                cprintln!("<green>{}</green>", chunk);
            }
            AncillaryChunk::pHYs(chunk) => {
                cprintln!("<green>{}</green>", chunk);
            }
            AncillaryChunk::Unknown(_) => {
                cprintln!("<green>Unknown chunk type</green>")
            }
        }
    }
}
