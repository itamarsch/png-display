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

        let mut read_value = || {
            let bytes = [remaining_input[0], remaining_input[1]];
            let pixel = u16::from_be_bytes(bytes);

            let value = if bit_depth <= 8 {
                map_pixel_value(bit_depth, pixel as u8)
            } else {
                (pixel >> 8) as u8
            };

            remaining_input = &remaining_input[2..];

            value
        };

        let color = match color_type {
            ColorType::Grayscale | ColorType::GrayscaleAlpha => {
                let grayscale = read_value();
                (grayscale, grayscale, grayscale)
            }
            ColorType::Rgb | ColorType::Rgba => {
                run_n!(3, read_value())
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
