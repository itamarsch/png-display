use crate::chunk::RawChunk;
use crate::filter_apply;
use crate::ihdr::{self, IhdrChunk};
use bitreader::BitReader;
use nom::{bytes::complete::tag, IResult};

use crate::chunk::parse_chunks;
use crate::ihdr::parse_ihdr;
use crate::plte::{self, parse_palette, Palette, PaletteEntries};

use crate::text::{CompressedTextChunk, InternationalTextChunk, TextChunk};

type Pixel = (u8, u8, u8, u8);
type Image = Vec<Vec<Pixel>>;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Chunk<'a> {
    tEXt(TextChunk<'a>),
    zTXt(CompressedTextChunk<'a>),
    iTXt(InternationalTextChunk<'a>),
    Unknown(RawChunk<'a>),
}

pub struct Png<'a> {
    pub ihdr: IhdrChunk,
    pub data: Vec<u8>,
    pub other_chunks: Vec<Chunk<'a>>,
}

fn take_palette_chunk(chunks: &mut Vec<RawChunk>) -> Option<Palette> {
    if let Some(i) = chunks.iter().position(|elem| elem.chunk_type == plte::PLTE) {
        let plte = chunks.remove(i);
        let trns = if let Some(i) = chunks.iter().position(|elem| elem.chunk_type == plte::TRNS) {
            let trns = chunks.remove(i);
            Some(trns.data)
        } else {
            None
        };

        let (_, palette) = parse_palette(plte.data, trns).unwrap();
        Some(palette)
    } else {
        None
    }
}

const IDAT: &str = "IDAT";
fn take_idta_chunks(chunks: &mut Vec<RawChunk>) -> Vec<u8> {
    let first_idat_index = chunks
        .iter()
        .position(|elem| elem.chunk_type == IDAT)
        .unwrap();

    let idat_indexes = chunks[first_idat_index..]
        .iter()
        .enumerate()
        .take_while(|(_, elem)| elem.chunk_type == IDAT)
        .map(|(i, _)| i + first_idat_index)
        .collect::<Vec<_>>();

    let data: Vec<u8> = idat_indexes
        .iter()
        .flat_map(|&index| chunks[index].data)
        .copied()
        .collect();

    // Remove IDAT chunks from the vector
    for &index in idat_indexes.iter().rev() {
        chunks.remove(index);
    }

    inflate::inflate_bytes_zlib(&data).unwrap()
}

fn parse_non_required_chunks(chunks: Vec<RawChunk<'_>>) -> Vec<Chunk<'_>> {
    chunks
        .into_iter()
        .map(|chunk| match chunk.chunk_type {
            TextChunk::CHUNK_TYPE => Chunk::tEXt(TextChunk::parse(chunk.data).unwrap()),
            CompressedTextChunk::CHUNK_TYPE => {
                Chunk::zTXt(CompressedTextChunk::parse(chunk.data).unwrap().1)
            }
            InternationalTextChunk::CHUNK_TYPE => {
                Chunk::iTXt(InternationalTextChunk::parse(chunk.data).unwrap().1)
            }
            _ => Chunk::Unknown(chunk),
        })
        .collect()
}

impl<'a> Png<'a> {
    pub fn new(input: &'a [u8]) -> IResult<&'a [u8], Self> {
        const MAGIC_NUMBER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

        let (input, _) = tag(MAGIC_NUMBER)(input)?;

        let (input, mut chunks) = parse_chunks(input)?;

        let palette = take_palette_chunk(&mut chunks);

        let ihdr = chunks.remove(0);
        let (_, ihdr) = parse_ihdr(ihdr.data, palette)?;

        let data = take_idta_chunks(&mut chunks);

        let non_requied_chunks = parse_non_required_chunks(chunks);
        Ok((
            input,
            Self {
                ihdr,
                data,
                other_chunks: non_requied_chunks,
            },
        ))
    }

    pub fn get_pixels(&self) -> anyhow::Result<Image> {
        match self.ihdr.interlace_method {
            ihdr::InterlaceMethod::Adam7 => self.get_pixels_adam7(),
            ihdr::InterlaceMethod::None => self.get_pixels_no_interlace(),
        }
    }

    fn read_pixel(&self, scanline_reader: &mut BitReader) -> anyhow::Result<Pixel> {
        let pixel = match &self.ihdr.color_type {
            ihdr::ColorType::Grayscale => {
                if self.ihdr.bit_depth == 8 {
                    let gray_scale = scanline_reader.read_u8(self.ihdr.bit_depth)?;
                    (gray_scale, gray_scale, gray_scale, 255)
                } else {
                    todo!()
                }
            }
            ihdr::ColorType::Rgb => todo!(),
            ihdr::ColorType::Palette(Palette {
                entries: PaletteEntries::RGBA(values),
            }) => {
                let index = scanline_reader.read_u8(self.ihdr.bit_depth)?;
                values[index as usize]
            }

            ihdr::ColorType::Palette(Palette {
                entries: PaletteEntries::RGB(values),
            }) => {
                let index = scanline_reader.read_u8(self.ihdr.bit_depth)?;
                let (r, g, b) = values[index as usize];
                (r, g, b, 255)
            }
            ihdr::ColorType::GrayscaleAlpha => todo!(),
            ihdr::ColorType::Rgba => {
                if self.ihdr.bit_depth == 8 {
                    let r = scanline_reader.read_u8(8)?;
                    let g = scanline_reader.read_u8(8)?;
                    let b = scanline_reader.read_u8(8)?;
                    let a = scanline_reader.read_u8(8)?;
                    (r, g, b, a)
                } else {
                    todo!()
                }
            }
        };
        Ok(pixel)
    }

    fn bpp(&self) -> usize {
        let values_per_pixel = self.ihdr.color_type.values_per_pixel();
        (self.ihdr.bit_depth.div_ceil(8) * values_per_pixel) as usize
    }

    fn get_pixels_no_interlace(&self) -> anyhow::Result<Image> {
        let mut bitreader = BitReader::new(&self.data[..]);

        let mut pixels =
            vec![vec![(0, 0, 0, 0); self.ihdr.width as usize]; self.ihdr.height as usize];
        let mut prev_scanline = None;

        let values_per_pixel = self.ihdr.color_type.values_per_pixel() as u32;
        let bits_in_scanline = self.ihdr.bit_depth as u32 * values_per_pixel * self.ihdr.width;
        let scanline_len = 1 + (bits_in_scanline as usize).div_ceil(8);

        let mut scanline = vec![0; scanline_len];
        let mut decoded = vec![0; scanline_len - 1];

        let bpp = self.bpp();
        for i in 0..self.ihdr.height {
            bitreader.read_u8_slice(&mut scanline)?;

            filter_apply::decode_scanline(
                &scanline[..],
                prev_scanline.as_ref().map(|v: &Vec<u8>| &v[..]),
                bpp,
                &mut decoded,
            );

            let mut scanline_reader = BitReader::new(&decoded);

            for j in 0..self.ihdr.width {
                pixels[i as usize][j as usize] = self.read_pixel(&mut scanline_reader)?;
            }

            prev_scanline = Some(decoded.clone());
        }

        Ok(pixels)
    }

    fn get_pixels_adam7(&self) -> anyhow::Result<Image> {
        let adam7_passes: [((usize, usize), (usize, usize)); 7] = [
            ((0, 0), (8, 8)),
            ((4, 0), (8, 8)),
            ((0, 4), (4, 8)),
            ((2, 0), (4, 4)),
            ((0, 2), (2, 4)),
            ((1, 0), (2, 2)),
            ((0, 1), (1, 2)),
        ];

        let mut bitreader = BitReader::new(&self.data[..]);

        let mut pixels =
            vec![vec![(0, 0, 0, 0); self.ihdr.width as usize]; self.ihdr.height as usize];
        let mut prev_scanline = None;

        for ((start_x, start_y), (step_x, step_y)) in adam7_passes {
            let scanline_len =
                1 + ((self.ihdr.width as usize - start_x).div_ceil(step_x)) * self.bpp();

            let mut scanline = vec![0; scanline_len];
            let mut decoded = vec![0; scanline_len - 1];
            for i in (start_y..self.ihdr.height as usize).step_by(step_y) {
                let bpp = self.bpp();
                bitreader.read_u8_slice(&mut scanline)?;

                filter_apply::decode_scanline(
                    &scanline[..],
                    prev_scanline.as_ref().map(|v: &Vec<u8>| &v[..]),
                    bpp,
                    &mut decoded,
                );

                let mut scanline_reader = BitReader::new(&decoded);
                for j in (start_x..self.ihdr.width as usize).step_by(step_x) {
                    pixels[i][j] = self.read_pixel(&mut scanline_reader)?;
                }
                prev_scanline = Some(decoded.clone());
            }
        }

        Ok(pixels)
    }
}
