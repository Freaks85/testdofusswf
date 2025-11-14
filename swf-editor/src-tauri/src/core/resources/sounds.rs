// Sound extraction
use crate::core::types::{SoundFormat, SoundResource};

/// Extract sound data in its native format
pub fn extract_sound(sound: &SoundResource) -> Result<Vec<u8>, String> {
    match sound.format {
        SoundFormat::Mp3 => {
            // MP3 data can be used directly
            Ok(sound.data.clone())
        }
        _ => {
            // Other formats may need conversion
            // For now, return raw data
            Ok(sound.data.clone())
        }
    }
}

/// Get file extension for sound format
pub fn get_extension(format: &SoundFormat) -> &'static str {
    match format {
        SoundFormat::Mp3 => "mp3",
        SoundFormat::Adpcm => "wav",
        SoundFormat::Uncompressed | SoundFormat::UncompressedLittleEndian => "pcm",
        _ => "bin",
    }
}
