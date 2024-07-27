use nom::number::complete::{be_u32, u8};
use nom::IResult;

use crate::plte::Palette;

#[derive(Debug)]
pub struct IhdrChunk {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

#[derive(Debug)]
pub enum ColorType {
    Grayscale,
    Rgb,
    Palette(Palette),
    GrayscaleAlpha,
    Rgba,
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
}

pub fn parse_ihdr(input: &[u8], plte: Option<Palette>) -> IResult<&[u8], IhdrChunk> {
    let (input, width) = be_u32(input)?;
    let (input, height) = be_u32(input)?;
    let (input, bit_depth) = u8(input)?;
    let (input, color_type_byte) = u8(input)?;
    let (input, compression_method) = u8(input)?;
    let (input, filter_method) = u8(input)?;
    let (input, interlace_method) = u8(input)?;

    let color_type = ColorType::from_u8(color_type_byte, plte).ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::NonEmpty,
        ))
    })?;

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
