use std::io::{self, Read};
use flate2::read::ZlibDecoder;
use lzma_rs::lzma_decompress;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("ZLIB decompression error: {0}")]
    ZlibError(String),

    #[error("LZMA decompression error: {0}")]
    LzmaError(String),

    #[error("Unknown compression format")]
    UnknownFormat,
}

/// Decompress ZLIB compressed data
pub fn decompress_zlib(compressed: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| CompressionError::ZlibError(e.to_string()))?;

    Ok(decompressed)
}

/// Decompress LZMA compressed data (used in ZWS files)
pub fn decompress_lzma(compressed: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut decompressed = Vec::new();
    let mut cursor = io::Cursor::new(compressed);

    lzma_decompress(&mut cursor, &mut decompressed)
        .map_err(|e| CompressionError::LzmaError(e.to_string()))?;

    Ok(decompressed)
}

/// Detect and decompress based on signature
pub fn decompress_swf_body(signature: &str, compressed: &[u8]) -> Result<Vec<u8>, CompressionError> {
    match signature {
        "FWS" => {
            // Uncompressed, return as-is
            Ok(compressed.to_vec())
        }
        "CWS" => {
            // ZLIB compressed
            decompress_zlib(compressed)
        }
        "ZWS" => {
            // LZMA compressed
            decompress_lzma(compressed)
        }
        _ => Err(CompressionError::UnknownFormat),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zlib_compression() {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let original = b"Hello, World! This is a test of ZLIB compression.";
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        let decompressed = decompress_zlib(&compressed).unwrap();
        assert_eq!(original, decompressed.as_slice());
    }
}
