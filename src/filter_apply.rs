#[derive(Debug)]
enum PngFilterType {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = a as isize + b as isize - c as isize;
    let pa = (p - a as isize).abs();
    let pb = (p - b as isize).abs();
    let pc = (p - c as isize).abs();

    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

pub fn decode_scanline(
    filtered_scanline: &[u8],
    previous_scanline: Option<&[u8]>,
    bytes_per_pixel: usize,
    decoded_scanline: &mut [u8],
) -> anyhow::Result<()> {
    let filter_type = match filtered_scanline[0] {
        0 => PngFilterType::None,
        1 => PngFilterType::Sub,
        2 => PngFilterType::Up,
        3 => PngFilterType::Average,
        4 => PngFilterType::Paeth,
        _ => anyhow::bail!("Invalid filter! {:?}", filtered_scanline[0]),
    };
    if !matches!(filter_type, PngFilterType::None) {
        // println!("{:?}", filter_type);
    }

    let filtered_scanline = &filtered_scanline[1..]; // Shift the slice to exclude the first byte

    match filter_type {
        PngFilterType::None => {
            decoded_scanline.copy_from_slice(filtered_scanline);
        }
        PngFilterType::Sub => {
            for i in 0..filtered_scanline.len() {
                if i < bytes_per_pixel {
                    decoded_scanline[i] = filtered_scanline[i];
                } else {
                    let decoded_byte =
                        filtered_scanline[i].wrapping_add(decoded_scanline[i - bytes_per_pixel]);
                    decoded_scanline[i] = decoded_byte;
                }
            }
        }
        PngFilterType::Up => {
            for i in 0..filtered_scanline.len() {
                let above = if let Some(previous) = previous_scanline {
                    previous[i]
                } else {
                    0
                };
                let decoded_byte = filtered_scanline[i].wrapping_add(above);
                decoded_scanline[i] = decoded_byte;
            }
        }
        PngFilterType::Average => {
            for i in 0..filtered_scanline.len() {
                let left = if i >= bytes_per_pixel {
                    decoded_scanline[i - bytes_per_pixel]
                } else {
                    0
                };
                let above = if let Some(previous) = previous_scanline {
                    previous[i]
                } else {
                    0
                };
                let decoded_byte =
                    filtered_scanline[i].wrapping_add(((left as u16 + above as u16) / 2) as u8);
                decoded_scanline[i] = decoded_byte;
            }
        }
        PngFilterType::Paeth => {
            for i in 0..filtered_scanline.len() {
                let left = if i >= bytes_per_pixel {
                    decoded_scanline[i - bytes_per_pixel]
                } else {
                    0
                };
                let above = if let Some(previous) = previous_scanline {
                    previous[i]
                } else {
                    0
                };
                let top_left = if let Some(previous) = previous_scanline {
                    if i >= bytes_per_pixel {
                        previous[i - bytes_per_pixel]
                    } else {
                        0
                    }
                } else {
                    0
                };
                let decoded_byte =
                    filtered_scanline[i].wrapping_add(paeth_predictor(left, above, top_left));
                decoded_scanline[i] = decoded_byte;
            }
        }
    };
    Ok(())
}
