pub mod compression;
pub mod header;
pub mod tags;

use crate::core::types::{Resources, SWFFile};
use std::fs;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Header error: {0}")]
    Header(#[from] header::HeaderError),

    #[error("Tag error: {0}")]
    Tag(#[from] tags::TagError),

    #[error("Compression error: {0}")]
    Compression(#[from] compression::CompressionError),

    #[error("File too small")]
    FileTooSmall,
}

/// Main function to parse a SWF file
pub fn parse_swf_file(path: &str) -> Result<SWFFile, ParseError> {
    // Read the entire file
    let file_data = fs::read(path)?;

    if file_data.len() < 8 {
        return Err(ParseError::FileTooSmall);
    }

    // Parse header from first 8 bytes
    let (header, header_size) = header::parse_header(&file_data)?;

    // Decompress the body if necessary
    let body_data = if header.compressed {
        // The first 8 bytes are uncompressed, rest is compressed
        let compressed_body = &file_data[8..];
        let decompressed = compression::decompress_swf_body(&header.signature, compressed_body)?;

        // Reconstruct: uncompressed header + remaining header data + decompressed body
        let mut full_data = Vec::new();
        full_data.extend_from_slice(&file_data[8..header_size]);
        full_data.extend_from_slice(&decompressed);
        full_data
    } else {
        // Already uncompressed, just take from after header
        file_data[8..].to_vec()
    };

    // Parse tags from the body
    let tags_data = &body_data[header_size - 8..]; // Adjust for the 8 bytes we already processed
    let tags = tags::parse_tags(tags_data)?;

    // Extract resources from tags
    let resources = extract_resources(&tags);

    Ok(SWFFile {
        path: path.to_string(),
        header,
        tags,
        resources,
    })
}

/// Extract resources from parsed tags
fn extract_resources(tags: &[crate::core::types::Tag]) -> Resources {
    use crate::core::types::*;

    let mut resources = Resources::default();

    for tag in tags {
        match tag {
            Tag::DefineBitsLossless {
                character_id,
                format,
                width,
                height,
                data,
            } => {
                resources.images.insert(
                    *character_id,
                    ImageResource {
                        id: *character_id,
                        width: *width,
                        height: *height,
                        format: ImageFormat::Png,
                        data: data.clone(),
                    },
                );
            }
            Tag::DefineBitsLossless2 {
                character_id,
                format,
                width,
                height,
                data,
            } => {
                resources.images.insert(
                    *character_id,
                    ImageResource {
                        id: *character_id,
                        width: *width,
                        height: *height,
                        format: ImageFormat::Png,
                        data: data.clone(),
                    },
                );
            }
            Tag::DefineBitsJPEG {
                character_id,
                image_data,
            } => {
                resources.images.insert(
                    *character_id,
                    ImageResource {
                        id: *character_id,
                        width: 0, // Unknown until decoded
                        height: 0,
                        format: ImageFormat::Jpeg,
                        data: image_data.clone(),
                    },
                );
            }
            Tag::DefineBitsJPEG2 {
                character_id,
                image_data,
            } => {
                resources.images.insert(
                    *character_id,
                    ImageResource {
                        id: *character_id,
                        width: 0,
                        height: 0,
                        format: ImageFormat::Jpeg,
                        data: image_data.clone(),
                    },
                );
            }
            Tag::DefineBitsJPEG3 {
                character_id,
                image_data,
                alpha_data,
            } => {
                resources.images.insert(
                    *character_id,
                    ImageResource {
                        id: *character_id,
                        width: 0,
                        height: 0,
                        format: ImageFormat::JpegWithAlpha,
                        data: image_data.clone(),
                    },
                );
            }
            Tag::DefineSound {
                character_id,
                format,
                rate,
                sound_type,
                sample_count,
                data,
                ..
            } => {
                let sound_format = match format {
                    0 => SoundFormat::Uncompressed,
                    1 => SoundFormat::Adpcm,
                    2 => SoundFormat::Mp3,
                    3 => SoundFormat::UncompressedLittleEndian,
                    4 => SoundFormat::Nellymoser16,
                    5 => SoundFormat::Nellymoser8,
                    6 => SoundFormat::Nellymoser,
                    11 => SoundFormat::Speex,
                    _ => SoundFormat::Uncompressed,
                };

                let sample_rate = match rate {
                    0 => 5512,
                    1 => 11025,
                    2 => 22050,
                    3 => 44100,
                    _ => 44100,
                };

                resources.sounds.insert(
                    *character_id,
                    SoundResource {
                        id: *character_id,
                        format: sound_format,
                        sample_rate,
                        stereo: *sound_type,
                        sample_count: *sample_count,
                        data: data.clone(),
                    },
                );
            }
            Tag::DefineSprite {
                character_id,
                frame_count,
                tags,
            } => {
                resources.sprites.insert(
                    *character_id,
                    SpriteResource {
                        id: *character_id,
                        frame_count: *frame_count,
                        tags: tags.clone(),
                    },
                );
            }
            Tag::DoABC {
                name, bytecode, ..
            } => {
                // Use a hash or counter for script ID since DoABC doesn't have character_id
                let script_id = (name.as_bytes().iter().map(|&b| b as u16).sum::<u16>()) % 10000;

                resources.scripts.insert(
                    script_id,
                    ScriptResource {
                        id: script_id,
                        name: name.clone(),
                        bytecode: bytecode.clone(),
                        decompiled: None,
                        script_type: ScriptType::AS3,
                    },
                );
            }
            Tag::DoAction { actions } => {
                // Similar approach for AS1/2 actions
                let script_id = (actions.iter().map(|&b| b as u16).sum::<u16>()) % 10000;

                resources.scripts.insert(
                    script_id,
                    ScriptResource {
                        id: script_id,
                        name: format!("Action_{}", script_id),
                        bytecode: actions.clone(),
                        decompiled: None,
                        script_type: ScriptType::AS1,
                    },
                );
            }
            Tag::DefineBinaryData {
                character_id,
                data,
            } => {
                resources.binary_data.insert(
                    *character_id,
                    BinaryDataResource {
                        id: *character_id,
                        data: data.clone(),
                    },
                );
            }
            Tag::DefineText {
                character_id,
                bounds,
                text_records,
                ..
            } | Tag::DefineText2 {
                character_id,
                bounds,
                text_records,
                ..
            } => {
                resources.texts.insert(
                    *character_id,
                    TextResource {
                        id: *character_id,
                        bounds: bounds.clone(),
                        text: format!("Text_{}", character_id), // Placeholder
                        raw_data: text_records.clone(),
                    },
                );
            }
            Tag::DefineEditText {
                character_id,
                bounds,
                data,
                ..
            } => {
                resources.texts.insert(
                    *character_id,
                    TextResource {
                        id: *character_id,
                        bounds: bounds.clone(),
                        text: format!("EditText_{}", character_id),
                        raw_data: data.clone(),
                    },
                );
            }
            Tag::DefineFont { character_id, data } => {
                resources.fonts.insert(
                    *character_id,
                    FontResource {
                        id: *character_id,
                        name: None,
                        num_glyphs: 0,
                        data: data.clone(),
                    },
                );
            }
            Tag::DefineFont2 {
                character_id,
                name,
                num_glyphs,
                data,
                ..
            } | Tag::DefineFont3 {
                character_id,
                name,
                num_glyphs,
                data,
                ..
            } => {
                resources.fonts.insert(
                    *character_id,
                    FontResource {
                        id: *character_id,
                        name: if name.is_empty() {
                            None
                        } else {
                            Some(name.clone())
                        },
                        num_glyphs: *num_glyphs,
                        data: data.clone(),
                    },
                );
            }
            Tag::DefineFont4 {
                character_id,
                name,
                data,
                ..
            } => {
                resources.fonts.insert(
                    *character_id,
                    FontResource {
                        id: *character_id,
                        name: if name.is_empty() {
                            None
                        } else {
                            Some(name.clone())
                        },
                        num_glyphs: 0,
                        data: data.clone(),
                    },
                );
            }
            Tag::DefineShape {
                character_id,
                bounds,
                shapes,
            } | Tag::DefineShape2 {
                character_id,
                bounds,
                shapes,
            } | Tag::DefineShape3 {
                character_id,
                bounds,
                shapes,
            } => {
                resources.shapes.insert(
                    *character_id,
                    ShapeResource {
                        id: *character_id,
                        bounds: bounds.clone(),
                        edge_bounds: None,
                        data: shapes.clone(),
                    },
                );
            }
            Tag::DefineShape4 {
                character_id,
                bounds,
                edge_bounds,
                shapes,
                ..
            } => {
                resources.shapes.insert(
                    *character_id,
                    ShapeResource {
                        id: *character_id,
                        bounds: bounds.clone(),
                        edge_bounds: Some(edge_bounds.clone()),
                        data: shapes.clone(),
                    },
                );
            }
            Tag::DoInitAction {
                sprite_id,
                actions,
            } => {
                resources.scripts.insert(
                    *sprite_id,
                    ScriptResource {
                        id: *sprite_id,
                        name: format!("InitAction_{}", sprite_id),
                        bytecode: actions.clone(),
                        decompiled: None,
                        script_type: ScriptType::AS2,
                    },
                );
            }
            _ => {
                // Other tags not yet handled for resource extraction
            }
        }
    }

    // Also recursively extract resources from sprite tags
    for tag in tags {
        if let Tag::DefineSprite { tags: sprite_tags, .. } = tag {
            let sprite_resources = extract_resources(sprite_tags);

            // Merge sprite resources into main resources
            for (id, img) in sprite_resources.images {
                resources.images.insert(id, img);
            }
            for (id, snd) in sprite_resources.sounds {
                resources.sounds.insert(id, snd);
            }
            for (id, spr) in sprite_resources.sprites {
                resources.sprites.insert(id, spr);
            }
            for (id, scr) in sprite_resources.scripts {
                resources.scripts.insert(id, scr);
            }
            for (id, txt) in sprite_resources.texts {
                resources.texts.insert(id, txt);
            }
            for (id, fnt) in sprite_resources.fonts {
                resources.fonts.insert(id, fnt);
            }
            for (id, shp) in sprite_resources.shapes {
                resources.shapes.insert(id, shp);
            }
            for (id, bin) in sprite_resources.binary_data {
                resources.binary_data.insert(id, bin);
            }
        }
    }

    resources
}
