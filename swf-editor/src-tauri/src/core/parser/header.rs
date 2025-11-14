use crate::core::types::{Rect, SWFHeader};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Cursor, Read};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeaderError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid SWF signature: {0}")]
    InvalidSignature(String),

    #[error("Invalid SWF version: {0}")]
    InvalidVersion(u8),

    #[error("Bit reading error: {0}")]
    BitReadError(String),
}

/// Parse the SWF file header (8 bytes + frame data)
pub fn parse_header(data: &[u8]) -> Result<(SWFHeader, usize), HeaderError> {
    if data.len() < 8 {
        return Err(HeaderError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "File too small to contain SWF header",
        )));
    }

    let mut cursor = Cursor::new(data);

    // Read signature (3 bytes)
    let mut sig_bytes = [0u8; 3];
    cursor.read_exact(&mut sig_bytes)?;
    let signature = String::from_utf8_lossy(&sig_bytes).to_string();

    // Validate signature
    if !matches!(signature.as_str(), "FWS" | "CWS" | "ZWS") {
        return Err(HeaderError::InvalidSignature(signature));
    }

    // Check if compressed
    let compressed = signature != "FWS";

    // Read version (1 byte)
    let version = cursor.read_u8()?;

    // Read file length (4 bytes, little-endian)
    let file_length = cursor.read_u32::<LittleEndian>()?;

    // The remaining data needs to be parsed from decompressed stream
    // For now, read the frame size, frame rate, and frame count from the remaining bytes
    let remaining = &data[8..];

    // Parse frame size (Rect structure - variable length bitfield)
    let (frame_size, rect_size) = parse_rect(remaining)?;

    // After rect, read frame rate (2 bytes, 8.8 fixed point)
    let frame_rate_bytes = &remaining[rect_size..rect_size + 2];
    let frame_rate_raw = u16::from_le_bytes([frame_rate_bytes[0], frame_rate_bytes[1]]);
    let frame_rate = (frame_rate_raw as f32) / 256.0;

    // Read frame count (2 bytes)
    let frame_count_bytes = &remaining[rect_size + 2..rect_size + 4];
    let frame_count = u16::from_le_bytes([frame_count_bytes[0], frame_count_bytes[1]]);

    let header_size = 8 + rect_size + 4; // 8 bytes header + rect + frame_rate (2) + frame_count (2)

    Ok((
        SWFHeader {
            signature,
            version,
            file_length,
            frame_size,
            frame_rate,
            frame_count,
            compressed,
        },
        header_size,
    ))
}

/// Parse a Rect structure (variable-length bitfield)
fn parse_rect(data: &[u8]) -> Result<(Rect, usize), HeaderError> {
    if data.is_empty() {
        return Err(HeaderError::BitReadError("No data for Rect".to_string()));
    }

    let mut bit_reader = BitReader::new(data);

    // First 5 bits indicate the number of bits for each coordinate
    let nbits = bit_reader
        .read_bits(5)
        .ok_or_else(|| HeaderError::BitReadError("Failed to read nbits".to_string()))? as usize;

    // Read each coordinate (x_min, x_max, y_min, y_max)
    let x_min = bit_reader
        .read_signed_bits(nbits)
        .ok_or_else(|| HeaderError::BitReadError("Failed to read x_min".to_string()))?;
    let x_max = bit_reader
        .read_signed_bits(nbits)
        .ok_or_else(|| HeaderError::BitReadError("Failed to read x_max".to_string()))?;
    let y_min = bit_reader
        .read_signed_bits(nbits)
        .ok_or_else(|| HeaderError::BitReadError("Failed to read y_min".to_string()))?;
    let y_max = bit_reader
        .read_signed_bits(nbits)
        .ok_or_else(|| HeaderError::BitReadError("Failed to read y_max".to_string()))?;

    let bytes_read = bit_reader.bytes_read();

    Ok((Rect::new(x_min, y_min, x_max, y_max), bytes_read))
}

/// Simple bit reader for parsing bitfield structures
struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    fn read_bits(&mut self, n: usize) -> Option<u32> {
        let mut result: u32 = 0;

        for _ in 0..n {
            if self.byte_pos >= self.data.len() {
                return None;
            }

            let bit = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1;
            result = (result << 1) | (bit as u32);

            self.bit_pos += 1;
            if self.bit_pos == 8 {
                self.bit_pos = 0;
                self.byte_pos += 1;
            }
        }

        Some(result)
    }

    fn read_signed_bits(&mut self, n: usize) -> Option<i32> {
        if n == 0 {
            return Some(0);
        }

        let unsigned = self.read_bits(n)?;

        // Check if sign bit is set
        if unsigned & (1 << (n - 1)) != 0 {
            // Negative number - sign extend
            let sign_extended = unsigned | (!0u32 << n);
            Some(sign_extended as i32)
        } else {
            Some(unsigned as i32)
        }
    }

    fn bytes_read(&self) -> usize {
        if self.bit_pos == 0 {
            self.byte_pos
        } else {
            self.byte_pos + 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_reader() {
        let data = vec![0b10110100, 0b11000000];
        let mut reader = BitReader::new(&data);

        assert_eq!(reader.read_bits(3), Some(0b101));
        assert_eq!(reader.read_bits(5), Some(0b10100));
        assert_eq!(reader.read_bits(2), Some(0b11));
    }

    #[test]
    fn test_signed_bits() {
        let data = vec![0b11100000]; // -1 in 3 bits (111)
        let mut reader = BitReader::new(&data);

        assert_eq!(reader.read_signed_bits(3), Some(-1));
    }
}
