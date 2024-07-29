use nom::{bytes::complete::take_until, number::complete::u8, IResult};

use crate::ihdr::CompressionMethod;

#[derive(Debug)]
pub struct TextChunk<'a> {
    pub keyword: &'a str,
    pub text: &'a str,
}
impl<'a> TextChunk<'a> {
    pub const CHUNK_TYPE: &'static str = "tEXt";
    pub fn parse(data: &'a [u8]) -> anyhow::Result<Self> {
        let s = data
            .split(|a| *a == 0)
            .map(|s| std::str::from_utf8(s))
            .collect::<Result<Vec<_>, _>>()?;

        if s.len() != 2 {
            anyhow::bail!("Invalid tEXt multiple null bytes");
        }
        Ok(TextChunk {
            keyword: s[0],
            text: s[1],
        })
    }
}

#[derive(Debug)]
pub struct CompressedTextChunk<'a> {
    pub keyword: &'a str,
    pub text: String,
}
impl<'a> CompressedTextChunk<'a> {
    pub const CHUNK_TYPE: &'static str = "zTXt";

    pub fn parse(input: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (input, keyword) = take_until(&[0][..])(input)?;
        let (input, _) = u8(input)?;
        let keyword = std::str::from_utf8(keyword).unwrap();

        let (input, compression_method) = u8(input)?;

        let CompressionMethod::Zlib = CompressionMethod::from_u8(compression_method).unwrap();

        let text = String::from_utf8(inflate::inflate_bytes_zlib(input).unwrap()).unwrap();

        Ok((input, CompressedTextChunk { text, keyword }))
    }
}

#[derive(Debug)]
pub struct InternationalTextChunk<'a> {
    pub keyword: &'a str,
    pub language_tag: &'a str,
    pub translated_keyword: &'a str,
    pub text: String,
}

enum CompressionFlags {
    Compression,
    NoCompression,
}
impl CompressionFlags {
    fn from_u8(value: u8) -> Option<CompressionFlags> {
        match value {
            0 => Some(CompressionFlags::NoCompression),
            1 => Some(CompressionFlags::Compression),
            _ => None,
        }
    }
}

impl<'a> InternationalTextChunk<'a> {
    pub const CHUNK_TYPE: &'static str = "iTXt";
    pub fn parse(input: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (input, keyword) = take_until(&[0][..])(input)?;
        let (input, _) = u8(input)?;

        let keyword = std::str::from_utf8(keyword).unwrap();
        let (input, compression_flags) = u8(input)?;
        let (input, compression_method) = u8(input)?;

        let CompressionMethod::Zlib = CompressionMethod::from_u8(compression_method).unwrap();
        let compression_flag = CompressionFlags::from_u8(compression_flags).unwrap();

        let (input, language_tag) = take_until(&[0][..])(input)?;
        let (input, _) = u8(input)?;
        let language_tag = std::str::from_utf8(language_tag).unwrap();

        let (input, translated) = take_until(&[0][..])(input)?;
        let (input, _) = u8(input)?;
        let translated_keyword = std::str::from_utf8(translated).unwrap();

        let text = match compression_flag {
            CompressionFlags::NoCompression => input.to_owned(),
            CompressionFlags::Compression => inflate::inflate_bytes_zlib(input).unwrap(),
        };

        let text = String::from_utf8(text).unwrap();
        Ok((
            input,
            InternationalTextChunk {
                keyword,
                language_tag,
                translated_keyword,
                text,
            },
        ))
    }
}
