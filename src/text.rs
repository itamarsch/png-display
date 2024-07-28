use nom::{bytes::complete::take_until, number::complete::u8, IResult};

pub fn parse_tEXt(data: &[u8]) -> anyhow::Result<(&str, &str)> {
    let s = data
        .split(|a| *a == 0)
        .map(|s| std::str::from_utf8(s))
        .collect::<Result<Vec<_>, _>>()?;

    if s.len() != 2 {
        anyhow::bail!("Invalid tEXt multiple null bytes");
    }
    Ok((s[0], s[1]))
}
pub fn parse_zTXt(input: &[u8]) -> IResult<&[u8], (&str, String)> {
    let (input, keyword) = take_until(&[0][..])(input)?;
    let (input, _) = u8(input)?;
    let keyword = std::str::from_utf8(keyword).unwrap();

    let (input, comression_method) = u8(input)?;
    assert_eq!(comression_method, 0);
    let text = String::from_utf8(inflate::inflate_bytes_zlib(input).unwrap()).unwrap();

    Ok((input, (keyword, text)))
}

#[derive(Debug)]
pub struct ITXT<'a> {
    pub keyword: &'a str,
    pub language_tag: &'a str,
    pub translated_keyword: &'a str,
    pub text: String,
}

pub fn parse_iTXt(input: &[u8]) -> IResult<&[u8], ITXT> {
    let (input, keyword) = take_until(&[0][..])(input)?;
    let (input, _) = u8(input)?;

    let keyword = std::str::from_utf8(keyword).unwrap();
    let (input, compression_flags) = u8(input)?;
    let (input, compression_method) = u8(input)?;
    assert_eq!(compression_method, 0);

    let (input, language_tag) = take_until(&[0][..])(input)?;
    let (input, _) = u8(input)?;
    let language_tag = std::str::from_utf8(language_tag).unwrap();

    let (input, translated) = take_until(&[0][..])(input)?;
    let (input, _) = u8(input)?;
    let translated_keyword = std::str::from_utf8(translated).unwrap();
    let text = if compression_flags == 0 {
        input.to_owned()
    } else {
        inflate::inflate_bytes_zlib(input).unwrap()
    };

    let text = String::from_utf8(text).unwrap();
    Ok((
        input,
        ITXT {
            keyword,
            language_tag,
            translated_keyword,
            text: text,
        },
    ))
}
