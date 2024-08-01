use anyhow::Context;
use nom::{
    number::{complete::u16, Endianness},
    IResult,
};

use crate::{
    color_type::{map_pixel_value, ColorType},
    plte::Palette,
    run_n,
};

#[derive(Debug)]
pub struct Background {
    pub color: (u8, u8, u8),
}

impl Background {
    pub const CHUNK_TYPE: &'static str = "bKGD";

    pub fn parse(input: &[u8], color_type: &ColorType, bit_depth: u8) -> anyhow::Result<Self> {
        let mut remaining_input = input;

        let mut read_value = || -> IResult<&[u8], u8> {
            let pixel;
            (remaining_input, pixel) = u16(Endianness::Big)(remaining_input)?;

            let value = if bit_depth <= 8 {
                map_pixel_value(bit_depth, pixel as u8)
            } else {
                (pixel >> 8) as u8
            };

            Ok((remaining_input, value))
        };

        let color = match color_type {
            ColorType::Grayscale { .. } | ColorType::GrayscaleAlpha => {
                let (_, grayscale) = read_value()
                    .map_err(|e| e.to_owned())
                    .context("Background parse grayscale")?;
                (grayscale, grayscale, grayscale)
            }
            ColorType::Rgb { .. } | ColorType::Rgba => {
                run_n!(3, read_value().map_err(|e| e.to_owned())?.1)
            }
            ColorType::Palette(Palette { entries }) => {
                let index = remaining_input[0] as usize;
                let pixel = entries[index];
                (pixel.0, pixel.1, pixel.2)
            }
        };

        Ok(Self { color })
    }
}
