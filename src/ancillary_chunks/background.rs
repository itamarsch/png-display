use bitreader::BitReader;

use crate::color_type::{read_and_map_u16, read_and_map_u8, ColorType};

#[derive(Debug)]
pub struct Background {
    pub color: (u8, u8, u8),
}

impl Background {
    pub const CHUNK_TYPE: &'static str = "bKGD";

    pub fn parse(input: &[u8], color_type: &ColorType, bit_depth: u8) -> Self {
        let mut remaining_input = input;

        let color = match color_type {
            ColorType::Grayscale | ColorType::GrayscaleAlpha => {
                let grayscale = {
                    let pixel = &remaining_input[0..2];
                    let mut reader = BitReader::new(pixel);

                    if bit_depth <= 8 {
                        let _ = reader.read_u8(8).unwrap();
                        println!("{:?}", pixel);
                        read_and_map_u8(bit_depth, &mut reader)
                    } else {
                        read_and_map_u16(bit_depth, &mut reader)
                    }
                    .unwrap()
                };
                (grayscale, grayscale, grayscale)
            }
            ColorType::Rgb | ColorType::Rgba => {
                let mut res = [0, 0, 0];
                (0..3).for_each(|i| {
                    let pixel = &remaining_input[0..2];
                    let mut reader = BitReader::new(pixel);
                    let value = if bit_depth <= 8 {
                        let _ = reader.read_u8(8).unwrap();
                        read_and_map_u8(bit_depth, &mut reader)
                    } else {
                        read_and_map_u16(bit_depth, &mut reader)
                    }
                    .unwrap();

                    remaining_input = &remaining_input[2..];
                    res[i] = value;
                });
                res.into()
            }
            ColorType::Palette(r) => {
                let mut reader = BitReader::new(remaining_input);
                let index = reader.read_u8(bit_depth).unwrap();
                match &r.entries {
                    crate::plte::PaletteEntries::RGB(rgb) => rgb[index as usize],
                    crate::plte::PaletteEntries::RGBA(rgba) => {
                        let pixel = rgba[index as usize];
                        (pixel.0, pixel.1, pixel.2)
                    }
                }
            }
        };
        println!("{:?}", color);

        Self { color }
    }
}
