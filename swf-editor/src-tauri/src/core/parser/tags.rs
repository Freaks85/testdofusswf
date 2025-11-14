use crate::core::types::{Rect, Tag};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Cursor, Read};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TagError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid tag format")]
    InvalidFormat,

    #[error("Unsupported tag type: {0}")]
    UnsupportedTag(u16),
}

/// Parse a single tag from the data
pub fn parse_tag(data: &[u8]) -> Result<(Tag, usize), TagError> {
    if data.len() < 2 {
        return Err(TagError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough data for tag header",
        )));
    }

    let mut cursor = Cursor::new(data);

    // Read tag code and length
    let tag_code_and_length = cursor.read_u16::<LittleEndian>()?;

    let tag_type = tag_code_and_length >> 6;
    let mut tag_length = (tag_code_and_length & 0x3F) as u32;

    // If length is 0x3F (63), read extended length
    if tag_length == 0x3F {
        tag_length = cursor.read_u32::<LittleEndian>()?;
    }

    let header_size = cursor.position() as usize;
    let tag_data_start = header_size;
    let tag_data_end = tag_data_start + tag_length as usize;

    if data.len() < tag_data_end {
        return Err(TagError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough data for tag content",
        )));
    }

    let tag_data = &data[tag_data_start..tag_data_end];

    // Parse specific tag types
    let tag = match tag_type {
        0 => Tag::End,
        9 => parse_set_background_color(tag_data)?,
        20 => parse_define_bits_lossless(tag_data, false)?,
        36 => parse_define_bits_lossless(tag_data, true)?,
        21 => parse_define_bits_jpeg(tag_data, 1)?,
        35 => parse_define_bits_jpeg(tag_data, 2)?,
        90 => parse_define_bits_jpeg(tag_data, 3)?,
        14 => parse_define_sound(tag_data)?,
        39 => parse_define_sprite(tag_data)?,
        82 => parse_do_abc(tag_data)?,
        12 => parse_do_action(tag_data)?,
        76 => parse_symbol_class(tag_data)?,
        87 => parse_define_binary_data(tag_data)?,
        _ => Tag::Unknown {
            tag_type,
            data: tag_data.to_vec(),
        },
    };

    Ok((tag, tag_data_end))
}

/// Parse all tags from data
pub fn parse_tags(data: &[u8]) -> Result<Vec<Tag>, TagError> {
    let mut tags = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        let (tag, size) = parse_tag(&data[pos..])?;

        let is_end = matches!(tag, Tag::End);
        tags.push(tag);

        if is_end {
            break;
        }

        pos += size;
    }

    Ok(tags)
}

fn parse_set_background_color(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 3 {
        return Err(TagError::InvalidFormat);
    }

    Ok(Tag::SetBackgroundColor {
        r: data[0],
        g: data[1],
        b: data[2],
    })
}

fn parse_define_bits_lossless(data: &[u8], with_alpha: bool) -> Result<Tag, TagError> {
    if data.len() < 7 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;
    let format = cursor.read_u8()?;
    let width = cursor.read_u16::<LittleEndian>()?;
    let height = cursor.read_u16::<LittleEndian>()?;

    let remaining = &data[7..];

    if with_alpha {
        Ok(Tag::DefineBitsLossless2 {
            character_id,
            format,
            width,
            height,
            data: remaining.to_vec(),
        })
    } else {
        Ok(Tag::DefineBitsLossless {
            character_id,
            format,
            width,
            height,
            data: remaining.to_vec(),
        })
    }
}

fn parse_define_bits_jpeg(data: &[u8], version: u8) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;

    let image_data = &data[2..];

    match version {
        1 => Ok(Tag::DefineBitsJPEG {
            character_id,
            image_data: image_data.to_vec(),
        }),
        2 => Ok(Tag::DefineBitsJPEG2 {
            character_id,
            image_data: image_data.to_vec(),
        }),
        3 => {
            // JPEG3 includes alpha data
            // First 4 bytes after character_id is the offset to alpha data
            if image_data.len() < 4 {
                return Err(TagError::InvalidFormat);
            }

            let alpha_offset =
                u32::from_le_bytes([image_data[0], image_data[1], image_data[2], image_data[3]])
                    as usize;
            let jpeg_data = &image_data[4..4 + alpha_offset];
            let alpha_data = &image_data[4 + alpha_offset..];

            Ok(Tag::DefineBitsJPEG3 {
                character_id,
                image_data: jpeg_data.to_vec(),
                alpha_data: alpha_data.to_vec(),
            })
        }
        _ => Err(TagError::UnsupportedTag(21)),
    }
}

fn parse_define_sound(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 7 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;
    let format_byte = cursor.read_u8()?;

    let format = (format_byte >> 4) & 0x0F;
    let rate = (format_byte >> 2) & 0x03;
    let size = (format_byte & 0x02) != 0;
    let sound_type = (format_byte & 0x01) != 0;

    let sample_count = cursor.read_u32::<LittleEndian>()?;

    let sound_data = &data[7..];

    Ok(Tag::DefineSound {
        character_id,
        format,
        rate,
        size,
        sound_type,
        sample_count,
        data: sound_data.to_vec(),
    })
}

fn parse_define_sprite(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 4 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;
    let frame_count = cursor.read_u16::<LittleEndian>()?;

    let sprite_tags = &data[4..];
    let tags = parse_tags(sprite_tags)?;

    Ok(Tag::DefineSprite {
        character_id,
        frame_count,
        tags,
    })
}

fn parse_do_abc(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 4 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let flags = cursor.read_u32::<LittleEndian>()?;

    // Read null-terminated name string
    let mut name_bytes = Vec::new();
    let mut pos = 4;

    while pos < data.len() && data[pos] != 0 {
        name_bytes.push(data[pos]);
        pos += 1;
    }

    let name = String::from_utf8_lossy(&name_bytes).to_string();
    pos += 1; // Skip null terminator

    let bytecode = &data[pos..];

    Ok(Tag::DoABC {
        flags,
        name,
        bytecode: bytecode.to_vec(),
    })
}

fn parse_do_action(data: &[u8]) -> Result<Tag, TagError> {
    Ok(Tag::DoAction {
        actions: data.to_vec(),
    })
}

fn parse_symbol_class(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let num_symbols = cursor.read_u16::<LittleEndian>()?;

    let mut symbols = Vec::new();
    let mut pos = 2;

    for _ in 0..num_symbols {
        if pos + 2 > data.len() {
            return Err(TagError::InvalidFormat);
        }

        let tag_id = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 2;

        // Read null-terminated name
        let mut name_bytes = Vec::new();
        while pos < data.len() && data[pos] != 0 {
            name_bytes.push(data[pos]);
            pos += 1;
        }
        pos += 1; // Skip null terminator

        let name = String::from_utf8_lossy(&name_bytes).to_string();
        symbols.push((tag_id, name));
    }

    Ok(Tag::SymbolClass { symbols })
}

fn parse_define_binary_data(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 6 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;
    let _reserved = cursor.read_u32::<LittleEndian>()?; // Reserved, must be 0

    let binary_data = &data[6..];

    Ok(Tag::DefineBinaryData {
        character_id,
        data: binary_data.to_vec(),
    })
}
