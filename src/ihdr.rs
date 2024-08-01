use anyhow::Context;
use nom::number::complete::{be_u32, u8};
use nom::IResult;

use crate::color_type::ColorType;
use crate::plte::Palette;

#[derive(Debug)]
pub struct IhdrChunk {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_method: CompressionMethod,
    pub filter_method: FilterMethod,
    pub interlace_method: InterlaceMethod,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterMethod {
    FiveFilter,
}

impl FilterMethod {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::FiveFilter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompressionMethod {
    Zlib,
}
impl CompressionMethod {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Zlib),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InterlaceMethod {
    Adam7,
    None,
}
impl InterlaceMethod {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Adam7),
            _ => None,
        }
    }
}

pub fn parse_ihdr<'a>(
    input: &'a [u8],

    plte: Option<Palette>,
    trns_content: Option<&'a [u8]>,
) -> anyhow::Result<(&'a [u8], IhdrChunk)> {
    type IHDRRaw = (u32, u32, u8, u8, u8, u8, u8);
    fn parse_nom(input: &[u8]) -> IResult<&[u8], IHDRRaw> {
        let (input, width) = be_u32(input)?;
        let (input, height) = be_u32(input)?;
        let (input, bit_depth) = u8(input)?;
        let (input, color_type) = u8(input)?;
        let (input, compression_method) = u8(input)?;
        let (input, filter_method) = u8(input)?;
        let (input, interlace_method) = u8(input)?;
        let ihdr_values = (
            width,
            height,
            bit_depth,
            color_type,
            compression_method,
            filter_method,
            interlace_method,
        );
        Ok((input, ihdr_values))
    }

    let (
        input,
        (width, height, bit_depth, color_type, compression_method, filter_method, interlace_method),
    ) = parse_nom(input).map_err(|e| e.to_owned())?;

    let color_type = ColorType::from_u8(color_type, bit_depth, plte, trns_content)?;
    let interlace_method = InterlaceMethod::from_u8(interlace_method)
        .context(format!("Invalid interlace_method: {}", interlace_method))?;

    let compression_method = CompressionMethod::from_u8(compression_method).context(format!(
        "Invalid compression_method: {}",
        compression_method
    ))?;

    let filter_method = FilterMethod::from_u8(filter_method)
        .context(format!("Invalid filter_method: {}", filter_method))?;

    let CompressionMethod::Zlib = compression_method;
    let FilterMethod::FiveFilter = filter_method;

    if !color_type.valid_bit_depths().contains(&bit_depth) {
        anyhow::bail!("Invalid bit_depth: {}", bit_depth)
    }

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
