use std::{
    borrow::Cow,
    fmt::{self, Display},
};

use nom::{bytes::complete::take_until, number::complete::u8, IResult};

use crate::ihdr::CompressionMethod;

fn iso_8859_1_to_string(bytes: &[u8]) -> Cow<str> {
    match std::str::from_utf8(bytes) {
        Ok(s) => Cow::Borrowed(s),
        Err(_) => Cow::Owned(bytes.iter().map(|&b| b as char).collect()),
    }
}
fn iso_8859_1_to_owned_string(bytes: Vec<u8>) -> String {
    bytes.into_iter().map(|s| s as char).collect()
}

#[derive(Debug)]
pub struct TextChunk<'a> {
    pub keyword: Cow<'a, str>,
    pub text: Cow<'a, str>,
}
impl<'a> TextChunk<'a> {
    pub const CHUNK_TYPE: &'static str = "tEXt";
    pub fn parse(data: &'a [u8]) -> anyhow::Result<Self> {
        let mut s = data
            .split(|a| *a == 0)
            .map(|s| iso_8859_1_to_string(s))
            .collect::<Vec<_>>();

        if s.len() != 2 {
            anyhow::bail!("Invalid tEXt multiple null bytes");
        }
        Ok(TextChunk {
            keyword: s.remove(0),
            text: s.remove(0),
        })
    }
}

#[derive(Debug)]
pub struct CompressedTextChunk<'a> {
    pub keyword: Cow<'a, str>,
    pub text: String,
}
impl<'a> CompressedTextChunk<'a> {
    pub const CHUNK_TYPE: &'static str = "zTXt";

    pub fn parse(input: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (input, keyword) = take_until(&[0][..])(input)?;
        let (input, _) = u8(input)?;
        let keyword = iso_8859_1_to_string(keyword);

        let (input, compression_method) = u8(input)?;

        let CompressionMethod::Zlib = CompressionMethod::from_u8(compression_method).unwrap();

        let text = iso_8859_1_to_owned_string(inflate::inflate_bytes_zlib(input).unwrap());

        Ok((input, CompressedTextChunk { text, keyword }))
    }
}

#[derive(Debug)]
pub struct InternationalTextChunk<'a> {
    pub keyword: Cow<'a, str>,
    pub language_tag: Cow<'a, str>,
    pub translated_keyword: Cow<'a, str>,
    pub text: Cow<'a, str>,
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

        let keyword = iso_8859_1_to_string(keyword);
        let (input, compression_flags) = u8(input)?;
        let (input, compression_method) = u8(input)?;

        let CompressionMethod::Zlib = CompressionMethod::from_u8(compression_method).unwrap();
        let compression_flag = CompressionFlags::from_u8(compression_flags).unwrap();

        let (input, language_tag) = take_until(&[0][..])(input)?;
        let (input, _) = u8(input)?;
        let language_tag = iso_8859_1_to_string(language_tag);

        let (input, translated) = take_until(&[0][..])(input)?;
        let (input, _) = u8(input)?;
        let translated_keyword = iso_8859_1_to_string(translated);

        let text = match compression_flag {
            CompressionFlags::NoCompression => iso_8859_1_to_string(input),
            CompressionFlags::Compression => Cow::Owned(iso_8859_1_to_owned_string(
                inflate::inflate_bytes_zlib(input).unwrap(),
            )),
        };

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

impl<'a> Display for TextChunk<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Keyword: {}\nText: {}", self.keyword, self.text)
    }
}

impl<'a> Display for CompressedTextChunk<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Keyword: {}\nText: {}", self.keyword, self.text)
    }
}

impl<'a> Display for InternationalTextChunk<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Keyword: {}\nLanguage Tag: {}\nTranslated Keyword: {}\nText: {}",
            self.keyword, self.language_tag, self.translated_keyword, self.text
        )
    }
}
