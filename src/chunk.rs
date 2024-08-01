use anyhow::Context;
use nom::{bytes::complete::take, number::complete::be_u32, IResult};

#[derive(Debug)]
pub struct RawChunk<'a> {
    pub chunk_type: &'a str,
    pub data: &'a [u8],
}

fn parse_chunk(input: &[u8]) -> anyhow::Result<(&[u8], RawChunk)> {
    type ChunkValues<'a> = (&'a [u8], &'a [u8], u32);
    fn parse_nom(input: &[u8]) -> IResult<&[u8], ChunkValues> {
        let (input, length) = be_u32(input)?;
        let (input, chunk_type) = take(4usize)(input)?;
        let (input, data) = take(length)(input)?;
        let (input, crc) = be_u32(input)?;
        Ok((input, (chunk_type, data, crc)))
    }

    let (input, (chunk_type, data, crc)) = parse_nom(input)
        .map_err(|e| e.to_owned())
        .context("Failed parsing chunk")?;

    let calculated_crc = calculate_crc(chunk_type, data);
    let chunk_type = std::str::from_utf8(chunk_type)?;
    if crc != calculated_crc {
        anyhow::bail!("Invalid crc in chunk: {:?}", chunk_type);
    }

    Ok((input, RawChunk { chunk_type, data }))
}

fn calculate_crc(chunk_type: &[u8], chunk_data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(chunk_type);
    hasher.update(chunk_data);
    hasher.finalize()
}

pub fn parse_chunks(input: &[u8]) -> anyhow::Result<Vec<RawChunk>> {
    let mut chunks = Vec::new();
    let mut remaining_input = input;

    while !remaining_input.is_empty() {
        let (rem, chunk) = parse_chunk(remaining_input)?;
        remaining_input = rem;
        chunks.push(chunk);
    }

    Ok(chunks)
}
