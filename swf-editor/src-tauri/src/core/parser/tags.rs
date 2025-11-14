use crate::core::types::{Rect, Tag};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Cursor};
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

/// Get human-readable tag name for debugging
fn get_tag_name(tag_type: u16) -> &'static str {
    match tag_type {
        0 => "End",
        1 => "ShowFrame",
        2 => "DefineShape",
        4 => "PlaceObject",
        5 => "RemoveObject",
        6 => "DefineBits",
        7 => "DefineButton",
        8 => "JPEGTables",
        9 => "SetBackgroundColor",
        10 => "DefineFont",
        11 => "DefineText",
        12 => "DoAction",
        13 => "DefineFontInfo",
        14 => "DefineSound",
        15 => "StartSound",
        17 => "DefineButtonSound",
        18 => "SoundStreamHead",
        19 => "SoundStreamBlock",
        20 => "DefineBitsLossless",
        21 => "DefineBitsJPEG2",
        22 => "DefineShape2",
        24 => "Protect",
        26 => "PlaceObject2",
        28 => "RemoveObject2",
        32 => "DefineShape3",
        33 => "DefineText2",
        34 => "DefineButton2",
        35 => "DefineBitsJPEG3",
        36 => "DefineBitsLossless2",
        37 => "DefineEditText",
        39 => "DefineSprite",
        43 => "FrameLabel",
        45 => "SoundStreamHead2",
        46 => "DefineMorphShape",
        48 => "DefineFont2",
        56 => "ExportAssets",
        57 => "ImportAssets",
        58 => "EnableDebugger",
        59 => "DoInitAction",
        60 => "DefineVideoStream",
        61 => "VideoFrame",
        62 => "DefineFontInfo2",
        63 => "DebugID",
        64 => "EnableDebugger2",
        65 => "ScriptLimits",
        66 => "SetTabIndex",
        69 => "FileAttributes",
        70 => "PlaceObject3",
        71 => "ImportAssets2",
        72 => "DoABC",
        73 => "DefineFontAlignZones",
        74 => "CSMTextSettings",
        75 => "DefineFont3",
        76 => "SymbolClass",
        77 => "Metadata",
        78 => "DefineScalingGrid",
        82 => "DoABC2",
        83 => "DefineShape4",
        84 => "DefineMorphShape2",
        86 => "DefineSceneAndFrameLabelData",
        87 => "DefineBinaryData",
        88 => "DefineFontName",
        89 => "StartSound2",
        90 => "DefineBitsJPEG4",
        91 => "DefineFont4",
        _ => "Unknown",
    }
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
            format!("Not enough data for tag {} ({}): need {}, have {}",
                    tag_type, get_tag_name(tag_type), tag_data_end, data.len()),
        )));
    }

    let tag_data = &data[tag_data_start..tag_data_end];

    // Parse specific tag types - use match with error recovery
    let tag = match tag_type {
        0 => Tag::End,
        1 => Tag::ShowFrame,
        9 => parse_set_background_color(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        20 => parse_define_bits_lossless(tag_data, false).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        36 => parse_define_bits_lossless(tag_data, true).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        21 => parse_define_bits_jpeg2(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        35 => parse_define_bits_jpeg3(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        90 => parse_define_bits_jpeg4(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        14 => parse_define_sound(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        39 => parse_define_sprite(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        82 | 72 => parse_do_abc(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        12 => parse_do_action(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        76 => parse_symbol_class(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        87 => parse_define_binary_data(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        26 => parse_place_object_2(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        70 => parse_place_object_3(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        28 => parse_remove_object_2(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        5 => parse_remove_object(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        43 => parse_frame_label(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        69 => parse_file_attributes(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        2 => parse_define_shape(tag_data, 1).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        22 => parse_define_shape(tag_data, 2).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        32 => parse_define_shape(tag_data, 3).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        83 => parse_define_shape(tag_data, 4).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        11 => parse_define_text(tag_data, 1).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        33 => parse_define_text(tag_data, 2).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        37 => parse_define_edit_text(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        56 => parse_export_assets(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        57 | 71 => parse_import_assets(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        59 => parse_do_init_action(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        77 => parse_metadata(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        65 => parse_script_limits(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        8 => Tag::JPEGTables { data: tag_data.to_vec() },
        18 => Tag::SoundStreamHead { data: tag_data.to_vec() },
        45 => Tag::SoundStreamHead2 { data: tag_data.to_vec() },
        19 => Tag::SoundStreamBlock { data: tag_data.to_vec() },
        15 | 89 => parse_start_sound(tag_data, tag_type).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        10 => parse_define_font(tag_data, 1).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        48 => parse_define_font(tag_data, 2).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        75 => parse_define_font(tag_data, 3).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        91 => parse_define_font(tag_data, 4).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        60 => parse_define_video_stream(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        61 => parse_video_frame(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        86 => parse_scene_and_frame_label_data(tag_data).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        7 | 34 => parse_define_button(tag_data, tag_type).unwrap_or_else(|_| create_unknown(tag_type, tag_data)),
        _ => create_unknown(tag_type, tag_data),
    };

    Ok((tag, tag_data_end))
}

/// Create an Unknown tag (helper for error recovery)
fn create_unknown(tag_type: u16, data: &[u8]) -> Tag {
    Tag::Unknown {
        tag_type,
        data: data.to_vec(),
    }
}

/// Parse all tags from data
pub fn parse_tags(data: &[u8]) -> Result<Vec<Tag>, TagError> {
    let mut tags = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        match parse_tag(&data[pos..]) {
            Ok((tag, size)) => {
                let is_end = matches!(tag, Tag::End);
                tags.push(tag);

                if is_end {
                    break;
                }

                pos += size;
            }
            Err(e) => {
                // Log error but continue parsing
                eprintln!("Error parsing tag at position {}: {}", pos, e);
                // Try to recover by skipping to next potential tag
                pos += 2; // Skip tag header
                if pos >= data.len() {
                    break;
                }
            }
        }
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

fn parse_define_bits_lossless(data: &[u8], has_alpha: bool) -> Result<Tag, TagError> {
    if data.len() < 7 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;
    let format = cursor.read_u8()?;
    let width = cursor.read_u16::<LittleEndian>()?;
    let height = cursor.read_u16::<LittleEndian>()?;

    let remaining_data = &data[7..];

    if has_alpha {
        Ok(Tag::DefineBitsLossless2 {
            character_id,
            format,
            width,
            height,
            data: remaining_data.to_vec(),
        })
    } else {
        Ok(Tag::DefineBitsLossless {
            character_id,
            format,
            width,
            height,
            data: remaining_data.to_vec(),
        })
    }
}

fn parse_define_bits_jpeg2(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;
    let image_data = data[2..].to_vec();

    Ok(Tag::DefineBitsJPEG2 {
        character_id,
        image_data,
    })
}

fn parse_define_bits_jpeg3(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 6 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;
    let alpha_data_offset = cursor.read_u32::<LittleEndian>()? as usize;

    let image_data_end = 6 + alpha_data_offset;
    if data.len() < image_data_end {
        return Err(TagError::InvalidFormat);
    }

    let image_data = data[6..image_data_end].to_vec();
    let alpha_data = data[image_data_end..].to_vec();

    Ok(Tag::DefineBitsJPEG3 {
        character_id,
        image_data,
        alpha_data,
    })
}

fn parse_define_bits_jpeg4(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 8 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;
    let alpha_data_offset = cursor.read_u32::<LittleEndian>()? as usize;
    let _deblocking_filter = cursor.read_u16::<LittleEndian>()?;

    let image_data_end = 8 + alpha_data_offset;
    if data.len() < image_data_end {
        return Err(TagError::InvalidFormat);
    }

    let image_data = data[8..image_data_end].to_vec();
    let alpha_data = data[image_data_end..].to_vec();

    Ok(Tag::DefineBitsJPEG3 { // Store as JPEG3 for compatibility
        character_id,
        image_data,
        alpha_data,
    })
}

fn parse_define_sound(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 7 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;

    let flags = cursor.read_u8()?;
    let format = (flags >> 4) & 0x0F;
    let rate = (flags >> 2) & 0x03;
    let size = (flags & 0x02) != 0;
    let sound_type = (flags & 0x01) != 0;

    let sample_count = cursor.read_u32::<LittleEndian>()?;

    let sound_data = data[7..].to_vec();

    Ok(Tag::DefineSound {
        character_id,
        format,
        rate,
        size,
        sound_type,
        sample_count,
        data: sound_data,
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

    let name = String::from_utf8(name_bytes).unwrap_or_else(|_| String::from(""));

    // Skip null terminator
    if pos < data.len() {
        pos += 1;
    }

    let bytecode = data[pos..].to_vec();

    Ok(Tag::DoABC {
        flags,
        name,
        bytecode,
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
            break;
        }

        let mut temp_cursor = Cursor::new(&data[pos..]);
        let tag_id = temp_cursor.read_u16::<LittleEndian>()?;
        pos += 2;

        // Read null-terminated string
        let mut name_bytes = Vec::new();
        while pos < data.len() && data[pos] != 0 {
            name_bytes.push(data[pos]);
            pos += 1;
        }

        let name = String::from_utf8(name_bytes).unwrap_or_else(|_| String::from(""));

        if pos < data.len() {
            pos += 1; // Skip null terminator
        }

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
    let _reserved = cursor.read_u32::<LittleEndian>()?; // Always 0

    let binary_data = data[6..].to_vec();

    Ok(Tag::DefineBinaryData {
        character_id,
        data: binary_data,
    })
}

fn parse_place_object_2(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 3 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let flags = cursor.read_u8()?;
    let depth = cursor.read_u16::<LittleEndian>()?;

    let has_character = (flags & 0x02) != 0;
    let has_matrix = (flags & 0x04) != 0;
    let has_color_transform = (flags & 0x08) != 0;
    let has_ratio = (flags & 0x10) != 0;
    let has_name = (flags & 0x20) != 0;
    let has_clip_depth = (flags & 0x40) != 0;

    let mut character_id = None;
    let mut matrix = None;
    let mut color_transform = None;
    let mut ratio = None;
    let mut name = None;
    let mut clip_depth = None;

    let mut pos = 3;

    if has_character && pos + 2 <= data.len() {
        character_id = Some(u16::from_le_bytes([data[pos], data[pos + 1]]));
        pos += 2;
    }

    if has_matrix {
        // Matrix is variable length - store rest of data for now
        matrix = Some(data[pos..].to_vec());
        // For simplicity, we'll store the rest as raw data
        return Ok(Tag::PlaceObject2 {
            flags,
            depth,
            character_id,
            matrix,
            color_transform,
            ratio,
            name,
            clip_depth,
        });
    }

    Ok(Tag::PlaceObject2 {
        flags,
        depth,
        character_id,
        matrix,
        color_transform,
        ratio,
        name,
        clip_depth,
    })
}

fn parse_place_object_3(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 4 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let flags = cursor.read_u16::<LittleEndian>()?;
    let depth = cursor.read_u16::<LittleEndian>()?;

    // For simplicity, store remaining data as raw
    Ok(Tag::PlaceObject3 {
        flags,
        depth,
        class_name: None,
        character_id: None,
        matrix: Some(data[4..].to_vec()),
        color_transform: None,
        ratio: None,
        name: None,
        clip_depth: None,
    })
}

fn parse_remove_object_2(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let depth = cursor.read_u16::<LittleEndian>()?;

    Ok(Tag::RemoveObject2 { depth })
}

fn parse_remove_object(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 4 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;
    let depth = cursor.read_u16::<LittleEndian>()?;

    Ok(Tag::RemoveObject { character_id, depth })
}

fn parse_frame_label(data: &[u8]) -> Result<Tag, TagError> {
    let mut name_bytes = Vec::new();
    let mut pos = 0;

    while pos < data.len() && data[pos] != 0 {
        name_bytes.push(data[pos]);
        pos += 1;
    }

    let name = String::from_utf8(name_bytes).unwrap_or_else(|_| String::from(""));

    // Check for anchor flag (SWF 6+)
    let anchor = if pos + 1 < data.len() {
        Some(data[pos + 1])
    } else {
        None
    };

    Ok(Tag::FrameLabel { name, anchor })
}

fn parse_file_attributes(data: &[u8]) -> Result<Tag, TagError> {
    Ok(Tag::FileAttributes {
        tag_id: 69,
        data: data.to_vec(),
    })
}

fn parse_define_shape(data: &[u8], version: u8) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;

    // Bounds is a RECT structure - variable length
    // For now, store all shape data as raw bytes
    let remaining = data[2..].to_vec();

    match version {
        1 => Ok(Tag::DefineShape {
            character_id,
            bounds: Rect::new(0, 0, 0, 0), // Placeholder
            shapes: remaining,
        }),
        2 => Ok(Tag::DefineShape2 {
            character_id,
            bounds: Rect::new(0, 0, 0, 0),
            shapes: remaining,
        }),
        3 => Ok(Tag::DefineShape3 {
            character_id,
            bounds: Rect::new(0, 0, 0, 0),
            shapes: remaining,
        }),
        4 => Ok(Tag::DefineShape4 {
            character_id,
            bounds: Rect::new(0, 0, 0, 0),
            edge_bounds: Rect::new(0, 0, 0, 0),
            flags: 0,
            shapes: remaining,
        }),
        _ => Err(TagError::InvalidFormat),
    }
}

fn parse_define_text(data: &[u8], version: u8) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;

    // Store rest as raw data for now
    let remaining = data[2..].to_vec();

    if version == 2 {
        Ok(Tag::DefineText2 {
            character_id,
            bounds: Rect::new(0, 0, 0, 0),
            matrix: Vec::new(),
            glyph_bits: 0,
            advance_bits: 0,
            text_records: remaining,
        })
    } else {
        Ok(Tag::DefineText {
            character_id,
            bounds: Rect::new(0, 0, 0, 0),
            matrix: Vec::new(),
            glyph_bits: 0,
            advance_bits: 0,
            text_records: remaining,
        })
    }
}

fn parse_define_edit_text(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;

    Ok(Tag::DefineEditText {
        character_id,
        bounds: Rect::new(0, 0, 0, 0),
        flags: 0,
        data: data[2..].to_vec(),
    })
}

fn parse_export_assets(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let count = cursor.read_u16::<LittleEndian>()?;

    let mut assets = Vec::new();
    let mut pos = 2;

    for _ in 0..count {
        if pos + 2 > data.len() {
            break;
        }

        let id = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 2;

        let mut name_bytes = Vec::new();
        while pos < data.len() && data[pos] != 0 {
            name_bytes.push(data[pos]);
            pos += 1;
        }

        let name = String::from_utf8(name_bytes).unwrap_or_default();
        if pos < data.len() {
            pos += 1;
        }

        assets.push((id, name));
    }

    Ok(Tag::ExportAssets { assets })
}

fn parse_import_assets(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 1 {
        return Err(TagError::InvalidFormat);
    }

    let mut pos = 0;
    let mut url_bytes = Vec::new();

    while pos < data.len() && data[pos] != 0 {
        url_bytes.push(data[pos]);
        pos += 1;
    }

    let url = String::from_utf8(url_bytes).unwrap_or_default();
    if pos < data.len() {
        pos += 1;
    }

    if pos + 2 > data.len() {
        return Ok(Tag::ImportAssets {
            url,
            assets: Vec::new(),
        });
    }

    let count = u16::from_le_bytes([data[pos], data[pos + 1]]);
    pos += 2;

    let mut assets = Vec::new();
    for _ in 0..count {
        if pos + 2 > data.len() {
            break;
        }

        let id = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 2;

        let mut name_bytes = Vec::new();
        while pos < data.len() && data[pos] != 0 {
            name_bytes.push(data[pos]);
            pos += 1;
        }

        let name = String::from_utf8(name_bytes).unwrap_or_default();
        if pos < data.len() {
            pos += 1;
        }

        assets.push((id, name));
    }

    Ok(Tag::ImportAssets { url, assets })
}

fn parse_do_init_action(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let sprite_id = cursor.read_u16::<LittleEndian>()?;

    Ok(Tag::DoInitAction {
        sprite_id,
        actions: data[2..].to_vec(),
    })
}

fn parse_metadata(data: &[u8]) -> Result<Tag, TagError> {
    let metadata = String::from_utf8(data.to_vec()).unwrap_or_default();
    Ok(Tag::Metadata { metadata })
}

fn parse_script_limits(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 4 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let max_recursion_depth = cursor.read_u16::<LittleEndian>()?;
    let script_timeout_seconds = cursor.read_u16::<LittleEndian>()?;

    Ok(Tag::ScriptLimits {
        max_recursion_depth,
        script_timeout_seconds,
    })
}

fn parse_start_sound(data: &[u8], tag_type: u16) -> Result<Tag, TagError> {
    if tag_type == 89 {
        // StartSound2 - has class name
        let mut pos = 0;
        let mut name_bytes = Vec::new();

        while pos < data.len() && data[pos] != 0 {
            name_bytes.push(data[pos]);
            pos += 1;
        }

        let class_name = String::from_utf8(name_bytes).unwrap_or_default();
        if pos < data.len() {
            pos += 1;
        }

        Ok(Tag::StartSound2 {
            sound_class_name: class_name,
            sound_info: data[pos..].to_vec(),
        })
    } else {
        // StartSound - has sound ID
        if data.len() < 2 {
            return Err(TagError::InvalidFormat);
        }

        let sound_id = u16::from_le_bytes([data[0], data[1]]);
        Ok(Tag::StartSound {
            sound_id,
            sound_info: data[2..].to_vec(),
        })
    }
}

fn parse_define_font(data: &[u8], version: u8) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;

    match version {
        1 => Ok(Tag::DefineFont {
            character_id,
            data: data[2..].to_vec(),
        }),
        2 => Ok(Tag::DefineFont2 {
            character_id,
            flags: 0,
            name: String::new(),
            num_glyphs: 0,
            data: data[2..].to_vec(),
        }),
        3 => Ok(Tag::DefineFont3 {
            character_id,
            flags: 0,
            name: String::new(),
            num_glyphs: 0,
            data: data[2..].to_vec(),
        }),
        4 => Ok(Tag::DefineFont4 {
            character_id,
            flags: 0,
            name: String::new(),
            data: data[2..].to_vec(),
        }),
        _ => Err(TagError::InvalidFormat),
    }
}

fn parse_define_video_stream(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 10 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;
    let num_frames = cursor.read_u16::<LittleEndian>()?;
    let width = cursor.read_u16::<LittleEndian>()?;
    let height = cursor.read_u16::<LittleEndian>()?;
    let flags = cursor.read_u8()?;
    let codec_id = cursor.read_u8()?;

    Ok(Tag::DefineVideoStream {
        character_id,
        num_frames,
        width,
        height,
        flags,
        codec_id,
    })
}

fn parse_video_frame(data: &[u8]) -> Result<Tag, TagError> {
    if data.len() < 4 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let stream_id = cursor.read_u16::<LittleEndian>()?;
    let frame_num = cursor.read_u16::<LittleEndian>()?;

    Ok(Tag::VideoFrame {
        stream_id,
        frame_num,
        video_data: data[4..].to_vec(),
    })
}

fn parse_scene_and_frame_label_data(data: &[u8]) -> Result<Tag, TagError> {
    // Complex structure - store as Unknown for now
    Ok(Tag::DefineSceneAndFrameLabelData {
        scenes: Vec::new(),
        frame_labels: Vec::new(),
    })
}

fn parse_define_button(data: &[u8], tag_type: u16) -> Result<Tag, TagError> {
    if data.len() < 2 {
        return Err(TagError::InvalidFormat);
    }

    let mut cursor = Cursor::new(data);
    let character_id = cursor.read_u16::<LittleEndian>()?;

    if tag_type == 34 {
        // DefineButton2
        let flags = if data.len() > 2 { data[2] } else { 0 };
        Ok(Tag::DefineButton2 {
            character_id,
            flags,
            data: data[3..].to_vec(),
        })
    } else {
        // DefineButton
        Ok(Tag::DefineButton {
            character_id,
            data: data[2..].to_vec(),
        })
    }
}
