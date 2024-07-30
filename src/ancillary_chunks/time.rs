use std::fmt;

use nom::number::complete::{u16, u8};
use nom::number::Endianness;
use nom::IResult;

#[derive(Debug)]
pub struct Time {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02}/{:02}/{:04} {:02}:{:02}:{:02}",
            self.day, self.month, self.year, self.hour, self.minute, self.second
        )
    }
}

fn assert_range<T>(value: T, top_limit: T, bottom: T)
where
    T: PartialOrd,
{
    assert!(value <= top_limit);
    assert!(bottom <= value);
}
impl Time {
    pub const CHUNK_TYPE: &'static str = "tIME";

    pub fn parse(input: &[u8]) -> IResult<&[u8], Time> {
        let (input, year) = u16(Endianness::Big)(input)?;
        let (input, month) = u8(input)?;
        let (input, day) = u8(input)?;
        let (input, hour) = u8(input)?;
        let (input, minute) = u8(input)?;
        let (input, second) = u8(input)?;
        assert_range(month, 12, 1);
        assert_range(day, 31, 1);
        assert_range(hour, 23, 0);
        assert_range(minute, 59, 0);
        assert_range(second, 60, 0);

        Ok((
            input,
            Time {
                year,
                month,
                day,
                hour,
                minute,
                second,
            },
        ))
    }
}
