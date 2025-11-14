// SWF writer module - for saving modified SWF files

use crate::core::types::{SWFFile, SWFHeader, Tag};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};
use std::fs::File;

pub fn write_swf(swf: &SWFFile, path: &str) -> Result<(), io::Error> {
    let mut buffer = Vec::new();

    // Write header
    write_header(&swf.header, &mut buffer)?;

    // Write tags
    write_tags(&swf.tags, &mut buffer)?;

    // Compress if needed
    let final_data = if swf.header.compressed {
        let mut compressed = Vec::new();
        // Write uncompressed header (first 8 bytes)
        compressed.write_all(&buffer[0..8])?;

        // Compress the rest
        let body = &buffer[8..];
        let compressed_body = compress_body(&swf.header.signature, body)?;
        compressed.write_all(&compressed_body)?;

        compressed
    } else {
        buffer
    };

    // Write to file
    let mut file = File::create(path)?;
    file.write_all(&final_data)?;

    Ok(())
}

fn write_header(header: &SWFHeader, buffer: &mut Vec<u8>) -> Result<(), io::Error> {
    // Signature (3 bytes)
    buffer.write_all(header.signature.as_bytes())?;

    // Version (1 byte)
    buffer.write_u8(header.version)?;

    // File length (4 bytes) - will be updated later
    let file_length_pos = buffer.len();
    buffer.write_u32::<LittleEndian>(header.file_length)?;

    // Frame size (Rect - variable length)
    write_rect(&header.frame_size, buffer)?;

    // Frame rate (2 bytes, 8.8 fixed point)
    let frame_rate_fixed = (header.frame_rate * 256.0) as u16;
    buffer.write_u16::<LittleEndian>(frame_rate_fixed)?;

    // Frame count (2 bytes)
    buffer.write_u16::<LittleEndian>(header.frame_count)?;

    Ok(())
}

fn write_rect(rect: &crate::core::types::Rect, buffer: &mut Vec<u8>) -> Result<(), io::Error> {
    // Calculate number of bits needed for coordinates
    let max_val = [rect.x_min.abs(), rect.x_max.abs(), rect.y_min.abs(), rect.y_max.abs()]
        .iter()
        .max()
        .unwrap_or(&0);

    let nbits = if *max_val == 0 {
        1
    } else {
        32 - max_val.leading_zeros() + 1 // +1 for sign bit
    } as u8;

    let nbits = nbits.min(31); // Max 31 bits

    // Write to bit buffer
    let mut bit_buffer: Vec<bool> = Vec::new();

    // Write nbits (5 bits)
    for i in (0..5).rev() {
        bit_buffer.push((nbits >> i) & 1 == 1);
    }

    // Write coordinates
    write_signed_bits(&mut bit_buffer, rect.x_min, nbits as usize);
    write_signed_bits(&mut bit_buffer, rect.x_max, nbits as usize);
    write_signed_bits(&mut bit_buffer, rect.y_min, nbits as usize);
    write_signed_bits(&mut bit_buffer, rect.y_max, nbits as usize);

    // Convert bit buffer to bytes
    let mut byte = 0u8;
    let mut bit_pos = 0;

    for bit in bit_buffer {
        if bit {
            byte |= 1 << (7 - bit_pos);
        }
        bit_pos += 1;

        if bit_pos == 8 {
            buffer.push(byte);
            byte = 0;
            bit_pos = 0;
        }
    }

    // Write remaining bits
    if bit_pos > 0 {
        buffer.push(byte);
    }

    Ok(())
}

fn write_signed_bits(buffer: &mut Vec<bool>, value: i32, nbits: usize) {
    let unsigned = if value < 0 {
        (1 << nbits) + value as u32
    } else {
        value as u32
    };

    for i in (0..nbits).rev() {
        buffer.push((unsigned >> i) & 1 == 1);
    }
}

fn write_tags(tags: &[Tag], buffer: &mut Vec<u8>) -> Result<(), io::Error> {
    for tag in tags {
        write_tag(tag, buffer)?;
    }

    // Write End tag
    buffer.write_u16::<LittleEndian>(0)?; // Tag type 0, length 0

    Ok(())
}

fn write_tag(tag: &Tag, buffer: &mut Vec<u8>) -> Result<(), io::Error> {
    // For now, write tags as Unknown with their raw data
    // This preserves the original data
    match tag {
        Tag::Unknown { tag_type, data } => {
            write_tag_header(*tag_type, data.len(), buffer)?;
            buffer.write_all(data)?;
        }
        _ => {
            // For structured tags, we need to serialize them back
            // For now, skip non-Unknown tags to avoid corruption
            // TODO: Implement proper serialization for all tag types
        }
    }

    Ok(())
}

fn write_tag_header(tag_type: u16, length: usize, buffer: &mut Vec<u8>) -> Result<(), io::Error> {
    if length < 63 {
        // Short header
        let header = (tag_type << 6) | (length as u16);
        buffer.write_u16::<LittleEndian>(header)?;
    } else {
        // Long header
        let header = (tag_type << 6) | 0x3F;
        buffer.write_u16::<LittleEndian>(header)?;
        buffer.write_u32::<LittleEndian>(length as u32)?;
    }

    Ok(())
}

fn compress_body(signature: &str, body: &[u8]) -> Result<Vec<u8>, io::Error> {
    match signature {
        "CWS" => {
            use flate2::write::ZlibEncoder;
            use flate2::Compression;

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(body)?;
            encoder.finish()
        }
        "ZWS" => {
            // LZMA compression
            Err(io::Error::new(
                io::ErrorKind::Other,
                "LZMA compression for saving not yet implemented",
            ))
        }
        _ => Ok(body.to_vec()), // FWS - no compression
    }
}
