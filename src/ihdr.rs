use nom::number::complete::{be_u32, u8};
use nom::IResult;

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

#[derive(Debug)]
pub enum ColorType {
    Grayscale,
    Rgb,
    Palette(Palette),
    GrayscaleAlpha,
    Rgba,
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

impl ColorType {
    fn from_u8(value: u8, plte: Option<Palette>) -> Option<ColorType> {
        match value {
            0 => Some(ColorType::Grayscale),
            2 => Some(ColorType::Rgb),
            3 => {
                let plte = plte?;
                Some(ColorType::Palette(plte))
            }
            4 => Some(ColorType::GrayscaleAlpha),
            6 => Some(ColorType::Rgba),
            _ => None,
        }
    }
    pub fn values_per_pixel(&self) -> u8 {
        match &self {
            ColorType::Grayscale => 1,
            ColorType::Rgb => 3,
            ColorType::Palette(_) => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::Rgba => 4,
        }
    }
}

pub fn parse_ihdr(input: &[u8], plte: Option<Palette>) -> IResult<&[u8], IhdrChunk> {
    let (input, width) = be_u32(input)?;
    let (input, height) = be_u32(input)?;
    let (input, bit_depth) = u8(input)?;
    let (input, color_type_byte) = u8(input)?;
    let (input, compression_method) = u8(input)?;
    let (input, filter_method) = u8(input)?;
    let (input, interlace_method) = u8(input)?;

    let color_type = ColorType::from_u8(color_type_byte, plte).unwrap();
    let interlace_method = InterlaceMethod::from_u8(interlace_method).unwrap();

    let compression_method = CompressionMethod::from_u8(compression_method).unwrap();
    let filter_method = FilterMethod::from_u8(filter_method).unwrap();

    let CompressionMethod::Zlib = compression_method;
    let FilterMethod::FiveFilter = filter_method;

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
