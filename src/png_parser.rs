use crate::ancillary_chunks::{parse_ancillary_chunks, AncillaryChunks};
use crate::chunk::RawChunk;
use crate::filter_apply;
use crate::ihdr::{self, IhdrChunk};
use anyhow::{anyhow, Context};
use bitreader::BitReader;
use nom::{bytes::complete::tag, IResult};

use crate::chunk::parse_chunks;
use crate::ihdr::parse_ihdr;
use crate::plte::{self, parse_palette, Palette};

pub type Pixel = (u8, u8, u8, u8);
pub type Image = Vec<Vec<Pixel>>;

pub const TRNS: &str = "tRNS";

pub struct Png<'a> {
    pub ihdr: IhdrChunk,
    pub data: Vec<u8>,
    pub other_chunks: AncillaryChunks<'a>,
}

fn take_chunk<'a>(chunks: &mut Vec<RawChunk<'a>>, chunk_type: &str) -> Option<RawChunk<'a>> {
    chunks
        .iter()
        .position(|elem| elem.chunk_type == chunk_type)
        .map(|i| chunks.remove(i))
}

fn take_palette_chunk(chunks: &mut Vec<RawChunk>) -> anyhow::Result<Option<Palette>> {
    let plte = take_chunk(chunks, plte::PLTE);
    if let Some(plte) = plte {
        let trns = take_chunk(chunks, TRNS).map(|c| c.data);
        let palette = parse_palette(plte.data, trns)?;
        Ok(Some(palette))
    } else {
        Ok(None)
    }
}

const IDAT: &str = "IDAT";
fn take_idta_chunks(chunks: &mut Vec<RawChunk>) -> anyhow::Result<Vec<u8>> {
    let first_idat_index = chunks
        .iter()
        .position(|elem| elem.chunk_type == IDAT)
        .context("No idat chunk")?;

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

    inflate::inflate_bytes_zlib(&data)
        .map_err(|e| anyhow!("Failed deompressing image data: {:?}", e))
}

impl<'a> Png<'a> {
    pub fn new(input: &'a [u8]) -> anyhow::Result<Self> {
        const MAGIC_NUMBER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

        fn read_magic_number(input: &[u8]) -> IResult<&[u8], ()> {
            let (input, _) = tag(MAGIC_NUMBER)(input)?;
            Ok((input, ()))
        }
        let (input, _) = read_magic_number(input).map_err(|e| e.to_owned())?;

        let mut chunks = parse_chunks(input)?;

        let palette = take_palette_chunk(&mut chunks)?;
        let trns = if palette.is_some() {
            None
        } else {
            take_chunk(&mut chunks, TRNS)
        }
        .map(|c| c.data);

        let ihdr = chunks.remove(0);
        let (_, ihdr) = parse_ihdr(ihdr.data, palette, trns)?;
        println!("{:?}", ihdr);

        let data = take_idta_chunks(&mut chunks)?;

        const IEND: &str = "IEND";
        let iend = chunks.remove(chunks.len() - 1);
        if iend.chunk_type != IEND {
            anyhow::bail!("Last chunk isn't IEND");
        }
        if !iend.data.is_empty() {
            anyhow::bail!("IEND isn't empty")
        }
        let non_requied_chunks = parse_ancillary_chunks(chunks, &ihdr)?;
        Ok(Self {
            ihdr,
            data,
            other_chunks: AncillaryChunks(non_requied_chunks),
        })
    }

    pub fn get_pixels(&self) -> anyhow::Result<Image> {
        match self.ihdr.interlace_method {
            ihdr::InterlaceMethod::Adam7 => self.get_pixels_adam7(),
            ihdr::InterlaceMethod::None => self.get_pixels_no_interlace(),
        }
    }

    pub fn print_ancillary(&self) {
        for chunk in &self.other_chunks.0 {
            chunk.print();
            println!();
        }
    }

    fn bpp(&self) -> usize {
        let values_per_pixel = self.ihdr.color_type.values_per_pixel();
        (self.ihdr.bit_depth.div_ceil(8) * values_per_pixel) as usize
    }
    fn bpp_float(&self) -> f32 {
        let values_per_pixel = self.ihdr.color_type.values_per_pixel() as f32;
        self.ihdr.bit_depth as f32 / 8.0 * values_per_pixel
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
            )?;

            let mut scanline_reader = BitReader::new(&decoded);

            for j in 0..self.ihdr.width {
                pixels[i as usize][j as usize] = self
                    .ihdr
                    .color_type
                    .read_pixel(self.ihdr.bit_depth, &mut scanline_reader)?;
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
            if (self.ihdr.width as usize) <= start_x || (self.ihdr.height as usize) <= start_y {
                continue;
            }
            let scanline_len = 1.0
                + (((self.ihdr.width as usize - start_x).div_ceil(step_x)) as f32
                    * self.bpp_float())
                .ceil();

            let scanline_len = scanline_len as usize;
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
                )?;

                let mut scanline_reader = BitReader::new(&decoded);
                for j in (start_x..self.ihdr.width as usize).step_by(step_x) {
                    pixels[i][j] = self
                        .ihdr
                        .color_type
                        .read_pixel(self.ihdr.bit_depth, &mut scanline_reader)?;
                }
                prev_scanline = Some(decoded.clone());
            }
        }

        Ok(pixels)
    }
}
