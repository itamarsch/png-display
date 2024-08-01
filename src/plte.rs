use anyhow::Context;
use nom::{number::complete::u8, IResult};

pub const PLTE: &str = "PLTE";

#[derive(Debug)]
pub struct Palette {
    pub entries: Vec<(u8, u8, u8, u8)>,
}

fn read_pixel(input: &[u8]) -> IResult<&[u8], (u8, u8, u8)> {
    let (input, red) = u8(input)?;
    let (input, green) = u8(input)?;
    let (input, blue) = u8(input)?;
    Ok((input, (red, green, blue)))
}

fn read_transparency(trns: &[u8]) -> IResult<&[u8], u8> {
    u8(trns)
}

pub fn parse_palette<'a>(input: &'a [u8], mut trns: Option<&'a [u8]>) -> anyhow::Result<Palette> {
    if let Some(trns) = trns {
        if trns.len() > input.len() / 3 {
            anyhow::bail!("Transparent chunk to long for palette")
        }
    }

    let mut palette = Vec::with_capacity(input.len() / 3);
    let mut remaining_input = input;

    while !remaining_input.is_empty() {
        let (input, (r, g, b)) = read_pixel(remaining_input)
            .map_err(|e| e.to_owned())
            .context("Failed parsing palette pixel")?;

        let transpareny = if let Some(trns) = trns.as_mut() {
            if trns.is_empty() {
                255
            } else {
                let (rest, transpareny) = read_transparency(trns)
                    .map_err(|e| e.to_owned())
                    .context("Failed parsing transpareny pixel")?;
                *trns = rest;

                transpareny
            }
        } else {
            255
        };

        palette.push((r, g, b, transpareny));
        remaining_input = input;
    }

    Ok(Palette { entries: palette })
}
