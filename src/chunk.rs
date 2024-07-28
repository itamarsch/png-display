use nom::{bytes::complete::take, number::complete::be_u32, IResult};

#[derive(Debug)]
pub struct PngChunk<'a> {
    pub chunk_type: &'a str,
    pub data: &'a [u8],
}

fn parse_chunk(input: &[u8]) -> IResult<&[u8], PngChunk> {
    let (input, length) = be_u32(input)?;
    let (input, chunk_type) = take(4usize)(input)?;
    let (input, data) = take(length)(input)?;
    let (input, crc) = be_u32(input)?;
    let calculated_crc = calculate_crc(chunk_type, data);
    if crc != calculated_crc {
        panic!("Invalid crc!!");
    }

    let chunk_type = std::str::from_utf8(chunk_type).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(
            chunk_type,
            nom::error::ErrorKind::Satisfy,
        ))
    })?;

    Ok((input, PngChunk { chunk_type, data }))
}

fn calculate_crc(chunk_type: &[u8], chunk_data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(chunk_type);
    hasher.update(chunk_data);
    hasher.finalize()
}

pub fn parse_chunks(input: &[u8]) -> IResult<&[u8], Vec<PngChunk>> {
    let mut chunks = Vec::new();
    let mut remaining_input = input;

    while !remaining_input.is_empty() {
        let (rem, chunk) = parse_chunk(remaining_input)?;
        remaining_input = rem;
        chunks.push(chunk);
    }

    Ok((remaining_input, chunks))
}
