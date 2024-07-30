use nom::{
    number::{complete::u16, Endianness},
    IResult,
};

use crate::{
    color_type::{map_pixel_value, ColorType},
    run_n,
};

#[derive(Debug)]
pub struct Background {
    pub color: (u8, u8, u8),
}

impl Background {
    pub const CHUNK_TYPE: &'static str = "bKGD";

    pub fn parse(input: &[u8], color_type: &ColorType, bit_depth: u8) -> Self {
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
                let (_, grayscale) = read_value().unwrap();
                (grayscale, grayscale, grayscale)
            }
            ColorType::Rgb { .. } | ColorType::Rgba => {
                run_n!(3, read_value().unwrap().1)
            }
            ColorType::Palette(r) => {
                let index = remaining_input[0] as usize;
                match &r.entries {
                    crate::plte::PaletteEntries::RGB(rgb) => rgb[index],
                    crate::plte::PaletteEntries::RGBA(rgba) => {
                        let pixel = rgba[index];
                        (pixel.0, pixel.1, pixel.2)
                    }
                }
            }
        };

        Self { color }
    }
}
