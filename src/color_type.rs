use bitreader::BitReader;
use seq_macro::seq;

use crate::{
    plte::{Palette, PaletteEntries},
    png_parser::Pixel,
};

macro_rules! run_n {
    ($n:expr, $expr:expr) => {
        {
            seq!(N in 0..$n {
                (
                    #($expr,)*
                )
            })
        }
    };
}

#[derive(Debug)]
pub enum ColorType {
    Grayscale,
    Rgb,
    Palette(Palette),
    GrayscaleAlpha,
    Rgba,
}
fn map_pixel_value(bit_depth: u8, value: u8) -> u8 {
    ((value as f32 / (2.0f32.powf(bit_depth as f32) - 1.0)) * 255.0) as u8
}

impl ColorType {
    pub fn valid_bit_depths(&self) -> Vec<u8> {
        match self {
            ColorType::Grayscale => vec![1, 2, 4, 8, 16],
            ColorType::Rgb => vec![8, 16],
            ColorType::Palette(_) => vec![1, 2, 4, 8],
            ColorType::GrayscaleAlpha => vec![8, 16],
            ColorType::Rgba => vec![8, 16],
        }
    }

    pub fn from_u8(value: u8, plte: Option<Palette>) -> Option<ColorType> {
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

    pub fn read_pixel(
        &self,
        bit_depth: u8,
        scanline_reader: &mut BitReader,
    ) -> anyhow::Result<Pixel> {
        let read_and_map_u8 = |scanline: &mut BitReader| {
            let v = scanline.read_u8(bit_depth)?;

            let v = if bit_depth != 8 {
                map_pixel_value(bit_depth, v)
            } else {
                v
            };
            let res: anyhow::Result<u8> = Ok(v);
            res
        };
        let read_and_map_u16 = |scanline: &mut BitReader| {
            let v = (scanline.read_u16(bit_depth)? >> 8) as u8;
            let res: anyhow::Result<u8> = Ok(v);
            res
        };

        let pixel = match &self {
            ColorType::Grayscale => {
                let grayscale = if bit_depth <= 8 {
                    read_and_map_u8(scanline_reader)?
                } else if bit_depth == 16 {
                    read_and_map_u16(scanline_reader)?
                } else {
                    unreachable!("Invalid bitdepth")
                };
                (grayscale, grayscale, grayscale, 255)
            }
            ColorType::Rgb => {
                let (r, g, b) = if bit_depth == 8 {
                    run_n!(3, scanline_reader.read_u8(8)?)
                } else if bit_depth == 16 {
                    run_n!(3, read_and_map_u16(scanline_reader)?)
                } else {
                    unreachable!("Invalid bitdepth")
                };

                (r, g, b, 255)
            }
            ColorType::Palette(Palette {
                entries: PaletteEntries::RGBA(values),
            }) => {
                if bit_depth <= 8 {
                    let index = scanline_reader.read_u8(bit_depth)?;
                    values[index as usize]
                } else {
                    unreachable!("Invalid bitdepth")
                }
            }

            ColorType::Palette(Palette {
                entries: PaletteEntries::RGB(values),
            }) => {
                if bit_depth <= 8 {
                    let index = scanline_reader.read_u8(bit_depth)?;
                    let (r, g, b) = values[index as usize];
                    (r, g, b, 255)
                } else {
                    unreachable!("Invalid bitdepth")
                }
            }
            ColorType::GrayscaleAlpha => {
                let (gray_scale, alpha) = if bit_depth == 8 {
                    run_n!(2, scanline_reader.read_u8(8)?)
                } else if bit_depth == 16 {
                    run_n!(2, read_and_map_u16(scanline_reader)?)
                } else {
                    unreachable!("Invalid bitdepth")
                };

                (gray_scale, gray_scale, gray_scale, alpha)
            }
            ColorType::Rgba => {
                if bit_depth == 8 {
                    run_n!(4, scanline_reader.read_u8(8)?)
                } else if bit_depth == 16 {
                    run_n!(4, read_and_map_u16(scanline_reader)?)
                } else {
                    unreachable!("Invalid bitdepth")
                }
            }
        };
        Ok(pixel)
    }
}
