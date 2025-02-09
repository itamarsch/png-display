use std::fmt;

use anyhow::Context;
use nom::{
    number::complete::{be_u32, be_u8},
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum UnitSpecifier {
    Unknown,
    Meter,
}

impl UnitSpecifier {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(UnitSpecifier::Unknown),
            1 => Some(UnitSpecifier::Meter),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PhysicalUnits {
    pub pixels_per_unit_x: u32,
    pub pixels_per_unit_y: u32,
    pub unit_specifier: UnitSpecifier,
    pub actual_width: f32,
    pub actual_height: f32,
}

impl PhysicalUnits {
    pub const CHUNK_TYPE: &'static str = "pHYs";

    pub fn parse(input: &[u8], width: u32, height: u32) -> anyhow::Result<PhysicalUnits> {
        fn parse_nom(input: &[u8]) -> IResult<&[u8], (u32, u32, u8)> {
            let (input, pixels_per_unit_x) = be_u32(input)?;
            let (input, pixels_per_unit_y) = be_u32(input)?;
            let (input, unit_specifier_byte) = be_u8(input)?;
            Ok((
                input,
                (pixels_per_unit_x, pixels_per_unit_y, unit_specifier_byte),
            ))
        }

        let (_, (pixels_per_unit_x, pixels_per_unit_y, unit_specifier_byte)) = parse_nom(input)
            .map_err(|err| err.to_owned())
            .context("pHYs chunk parsing")?;

        let unit_specifier = UnitSpecifier::from_u8(unit_specifier_byte)
            .context("Unit specifier in phys chunk invalid")?;

        let actual_width = width as f32 / pixels_per_unit_x as f32;
        let actual_height = height as f32 / pixels_per_unit_y as f32;

        Ok(PhysicalUnits {
            pixels_per_unit_x,
            pixels_per_unit_y,
            unit_specifier,
            actual_width,
            actual_height,
        })
    }
}
impl fmt::Display for PhysicalUnits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit = match self.unit_specifier {
            UnitSpecifier::Meter => "m",
            UnitSpecifier::Unknown => "",
        };
        write!(
            f,
            "Physical Ratio: {:.2}{} x {:.2}{}",
            self.actual_width, unit, self.actual_height, unit
        )
    }
}
