use nom::{number::complete::u8, IResult};

pub const PLTE: &str = "PLTE";
pub const TRNS: &str = "tRNS";

#[derive(Debug)]
pub enum PaletteEntries {
    RGB(Vec<(u8, u8, u8)>),
    RGBA(Vec<(u8, u8, u8, u8)>),
}

#[derive(Debug)]
pub struct Palette {
    pub entries: PaletteEntries,
}

pub fn parse_palette<'a>(input: &'a [u8], trns: Option<&'a [u8]>) -> IResult<&'a [u8], Palette> {
    match trns {
        Some(trns) => {
            if trns.len() > input.len() / 3 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::TooLarge,
                )));
            }
            let mut palette = Vec::with_capacity(input.len() / 3);
            let mut remaining_input = input;
            let mut trns = trns;

            while !remaining_input.is_empty() {
                let (input, red) = u8(remaining_input)?;
                let (input, green) = u8(input)?;
                let (input, blue) = u8(input)?;

                let (rem_trns, transpareny) = if !trns.is_empty() {
                    u8(trns)?
                } else {
                    (trns, 255)
                };
                trns = rem_trns;

                palette.push((red, green, blue, transpareny));
                remaining_input = input;
            }

            Ok((
                remaining_input,
                Palette {
                    entries: PaletteEntries::RGBA(palette),
                },
            ))
        }
        None => {
            let mut palette = Vec::with_capacity(input.len() / 3);
            let mut remaining_input = input;

            while !remaining_input.is_empty() {
                let (input, red) = u8(remaining_input)?;
                let (input, green) = u8(input)?;
                let (input, blue) = u8(input)?;

                palette.push((red, green, blue));
                remaining_input = input;
            }

            Ok((
                remaining_input,
                Palette {
                    entries: PaletteEntries::RGB(palette),
                },
            ))
        }
    }
}
