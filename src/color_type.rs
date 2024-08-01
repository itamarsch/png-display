use anyhow::Context;
use bitreader::BitReader;

use crate::{plte::Palette, png_parser::Pixel, run_n};

#[derive(Debug)]
pub enum ColorType {
    Grayscale { transparent: Option<u8> },
    Rgb { transparent: Option<(u8, u8, u8)> },
    Palette(Palette),
    GrayscaleAlpha,
    Rgba,
}
pub fn map_pixel_value(bit_depth: u8, value: u8) -> u8 {
    ((value as f32 / (2.0f32.powf(bit_depth as f32) - 1.0)) * 255.0) as u8
}

impl ColorType {
    pub fn valid_bit_depths(&self) -> Vec<u8> {
        match self {
            ColorType::Grayscale { .. } => vec![1, 2, 4, 8, 16],
            ColorType::Rgb { .. } => vec![8, 16],
            ColorType::Palette(_) => vec![1, 2, 4, 8],
            ColorType::GrayscaleAlpha => vec![8, 16],
            ColorType::Rgba => vec![8, 16],
        }
    }

    pub fn from_u8(
        value: u8,
        bit_depth: u8,
        plte: Option<Palette>,
        trns_content: Option<&[u8]>,
    ) -> anyhow::Result<ColorType> {
        let read_trns_value = |buf: [u8; 2]| -> anyhow::Result<u8> {
            let v = u16::from_be_bytes(buf);
            if bit_depth <= 8 {
                Ok(map_pixel_value(bit_depth, v as u8))
            } else {
                Ok((v >> 8) as u8)
            }
        };
        match value {
            0 => {
                let transparent = if let Some(trns_content) = trns_content {
                    if trns_content.len() != 2 {
                        anyhow::bail!("Invalid transparent buffer len")
                    }
                    let buf = (&trns_content[..2])
                        .try_into()
                        .expect("Slice len validated");

                    Some(read_trns_value(buf)?)
                } else {
                    None
                };

                Ok(ColorType::Grayscale { transparent })
            }
            2 => {
                let pixel = if let Some(trns_content) = trns_content {
                    if trns_content.len() != 6 {
                        anyhow::bail!("Invalid transparent buffer len")
                    }

                    let r =
                        read_trns_value((&trns_content[..2]).try_into().expect("Len validated"))?;
                    let g =
                        read_trns_value((&trns_content[2..4]).try_into().expect("Len validated"))?;
                    let b =
                        read_trns_value((&trns_content[4..6]).try_into().expect("Len validated"))?;
                    Some((r, g, b))
                } else {
                    None
                };

                Ok(ColorType::Rgb { transparent: pixel })
            }
            3 => {
                let plte = plte.context("Expected palette for ColorType 3 but no one was found")?;
                Ok(ColorType::Palette(plte))
            }
            4 => Ok(ColorType::GrayscaleAlpha),
            6 => Ok(ColorType::Rgba),
            e => Err(anyhow::anyhow!("Invalid color type: {}", e)),
        }
    }
    pub fn values_per_pixel(&self) -> u8 {
        match &self {
            ColorType::Grayscale { .. } => 1,
            ColorType::Rgb { .. } => 3,
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
        let read_and_map_u8 = |scanline: &mut BitReader| -> anyhow::Result<u8> {
            let v = scanline.read_u8(bit_depth)?;

            let v = if bit_depth != 8 {
                map_pixel_value(bit_depth, v)
            } else {
                v
            };
            Ok(v)
        };

        let read_and_map_u16 = |scanline: &mut BitReader| -> anyhow::Result<u8> {
            let v = (scanline.read_u16(bit_depth)? >> 8) as u8;
            Ok(v)
        };

        let pixel = match &self {
            ColorType::Grayscale { transparent } => {
                let grayscale = if bit_depth <= 8 {
                    read_and_map_u8(scanline_reader)?
                } else if bit_depth == 16 {
                    read_and_map_u16(scanline_reader)?
                } else {
                    unreachable!("Invalid bitdepth")
                };

                (
                    grayscale,
                    grayscale,
                    grayscale,
                    if let Some(transparent) = transparent {
                        if *transparent == grayscale {
                            0
                        } else {
                            255
                        }
                    } else {
                        255
                    },
                )
            }
            ColorType::Rgb { transparent } => {
                let pixel = if bit_depth == 8 {
                    run_n!(3, scanline_reader.read_u8(8)?)
                } else if bit_depth == 16 {
                    run_n!(3, read_and_map_u16(scanline_reader)?)
                } else {
                    unreachable!("Invalid bitdepth")
                };
                let alpha = if let Some(transparent) = transparent {
                    if *transparent == pixel {
                        0
                    } else {
                        255
                    }
                } else {
                    255
                };

                let (r, g, b) = pixel;
                (r, g, b, alpha)
            }
            ColorType::Palette(Palette { entries }) => {
                if bit_depth <= 8 {
                    let index = scanline_reader.read_u8(bit_depth)?;
                    entries[index as usize]
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
