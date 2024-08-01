use std::{
    borrow::Cow,
    fmt::{self, Display},
};

use anyhow::{anyhow, Context};
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

    pub fn parse(input: &'a [u8]) -> anyhow::Result<CompressedTextChunk> {
        fn parse_nom(input: &[u8]) -> IResult<&[u8], (Cow<str>, u8)> {
            let (input, keyword) = take_until(&[0][..])(input)?;
            let keyword = iso_8859_1_to_string(keyword);
            let (input, _) = u8(input)?;
            let (input, compression_method) = u8(input)?;
            Ok((input, (keyword, compression_method)))
        }

        let (compressed, (keyword, compression_method)) = parse_nom(input)
            .map_err(|e| e.to_owned())
            .context("zTXt chunk parsing")?;

        let CompressionMethod::Zlib = CompressionMethod::from_u8(compression_method)
            .context("Compression method value zTXt")?;

        let text = iso_8859_1_to_owned_string(
            inflate::inflate_bytes_zlib(compressed)
                .map_err(|e| anyhow!("Failed decompression the zTXt chunk: {:?}", e))?,
        );

        Ok(CompressedTextChunk { text, keyword })
    }
}

#[derive(Debug)]
pub struct InternationalTextChunk<'a> {
    pub keyword: &'a str,
    pub language_tag: &'a str,
    pub translated_keyword: &'a str,
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
    pub fn parse(input: &'a [u8]) -> anyhow::Result<Self> {
        type ITXTRaw<'a> = (&'a [u8], u8, u8, &'a [u8], &'a [u8]);
        fn parse_nom(input: &[u8]) -> IResult<&[u8], ITXTRaw> {
            let (input, keyword) = take_until(&[0][..])(input)?;
            let (input, _) = u8(input)?;

            let (input, compression_flags) = u8(input)?;
            let (input, compression_method) = u8(input)?;

            let (input, language_tag) = take_until(&[0][..])(input)?;
            let (input, _) = u8(input)?;

            let (input, translated) = take_until(&[0][..])(input)?;
            let (input, _) = u8(input)?;
            Ok((
                input,
                (
                    keyword,
                    compression_flags,
                    compression_method,
                    language_tag,
                    translated,
                ),
            ))
        }

        let (input, (keyword, compression_flags, compression_method, language_tag, translated)) =
            parse_nom(input)
                .map_err(|e| e.to_owned())
                .context("iTXt parsing")?;

        let CompressionMethod::Zlib =
            CompressionMethod::from_u8(compression_method).context("CompressionMethod from_u8")?;

        let compression_flag =
            CompressionFlags::from_u8(compression_flags).context("CompressionFlags from_u8")?;
        let keyword = std::str::from_utf8(keyword)?;
        let language_tag = std::str::from_utf8(language_tag)?;
        let translated_keyword = std::str::from_utf8(translated)?;

        let text = match compression_flag {
            CompressionFlags::NoCompression => Cow::Borrowed(std::str::from_utf8(input)?),
            CompressionFlags::Compression => Cow::Owned(String::from_utf8(
                inflate::inflate_bytes_zlib(input)
                    .map_err(|e| anyhow!("Failed decompressing iTXt: {:?}", e))?,
            )?),
        };

        Ok(InternationalTextChunk {
            keyword,
            language_tag,
            translated_keyword,
            text,
        })
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
