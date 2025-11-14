// Image extraction and conversion
use crate::core::types::{ImageFormat, ImageResource};
use std::io::Cursor;

/// Convert SWF image data to PNG format
pub fn to_png(image: &ImageResource) -> Result<Vec<u8>, String> {
    match image.format {
        ImageFormat::Png => {
            // Already PNG, might need decompression
            decompress_image_data(&image.data, image.width, image.height)
        }
        ImageFormat::Jpeg => {
            // Return JPEG as-is
            Ok(image.data.clone())
        }
        ImageFormat::JpegWithAlpha => {
            // TODO: Combine JPEG with alpha channel
            Ok(image.data.clone())
        }
    }
}

fn decompress_image_data(data: &[u8], width: u16, height: u16) -> Result<Vec<u8>, String> {
    // DefineBitsLossless data is ZLIB compressed
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("ZLIB decompression error: {}", e))?;

    // TODO: Convert decompressed bitmap data to proper PNG format
    // For now, just return the decompressed data
    Ok(decompressed)
}
