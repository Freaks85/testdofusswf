use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main SWF file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SWFFile {
    pub path: String,
    pub header: SWFHeader,
    pub tags: Vec<Tag>,
    pub resources: Resources,
}

/// SWF file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SWFHeader {
    pub signature: String,      // "FWS", "CWS", "ZWS"
    pub version: u8,
    pub file_length: u32,
    pub frame_size: Rect,
    pub frame_rate: f32,
    pub frame_count: u16,
    pub compressed: bool,
}

/// Rectangle structure (used for bounds, frame size, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x_min: i32,
    pub x_max: i32,
    pub y_min: i32,
    pub y_max: i32,
}

impl Rect {
    pub fn new(x_min: i32, y_min: i32, x_max: i32, y_max: i32) -> Self {
        Self {
            x_min,
            y_min,
            x_max,
            y_max,
        }
    }

    pub fn width(&self) -> i32 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> i32 {
        self.y_max - self.y_min
    }
}

/// SWF Tag types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Tag {
    FileAttributes {
        tag_id: u16,
        data: Vec<u8>,
    },
    SetBackgroundColor {
        r: u8,
        g: u8,
        b: u8,
    },
    DefineBitsLossless {
        character_id: u16,
        format: u8,
        width: u16,
        height: u16,
        data: Vec<u8>,
    },
    DefineBitsLossless2 {
        character_id: u16,
        format: u8,
        width: u16,
        height: u16,
        data: Vec<u8>,
    },
    DefineBitsJPEG {
        character_id: u16,
        image_data: Vec<u8>,
    },
    DefineBitsJPEG2 {
        character_id: u16,
        image_data: Vec<u8>,
    },
    DefineBitsJPEG3 {
        character_id: u16,
        image_data: Vec<u8>,
        alpha_data: Vec<u8>,
    },
    DefineSound {
        character_id: u16,
        format: u8,
        rate: u8,
        size: bool,
        sound_type: bool,
        sample_count: u32,
        data: Vec<u8>,
    },
    DefineSprite {
        character_id: u16,
        frame_count: u16,
        tags: Vec<Tag>,
    },
    DoABC {
        flags: u32,
        name: String,
        bytecode: Vec<u8>,
    },
    DoAction {
        actions: Vec<u8>,
    },
    PlaceObject {
        character_id: Option<u16>,
        depth: u16,
        matrix: Option<Vec<u8>>,
        color_transform: Option<Vec<u8>>,
    },
    PlaceObject2 {
        flags: u8,
        depth: u16,
        character_id: Option<u16>,
        matrix: Option<Vec<u8>>,
        color_transform: Option<Vec<u8>>,
        ratio: Option<u16>,
        name: Option<String>,
        clip_depth: Option<u16>,
    },
    PlaceObject3 {
        flags: u16,
        depth: u16,
        class_name: Option<String>,
        character_id: Option<u16>,
        matrix: Option<Vec<u8>>,
        color_transform: Option<Vec<u8>>,
        ratio: Option<u16>,
        name: Option<String>,
        clip_depth: Option<u16>,
    },
    DefineText {
        character_id: u16,
        bounds: Rect,
        matrix: Vec<u8>,
        glyph_bits: u8,
        advance_bits: u8,
        text_records: Vec<u8>,
    },
    DefineText2 {
        character_id: u16,
        bounds: Rect,
        matrix: Vec<u8>,
        glyph_bits: u8,
        advance_bits: u8,
        text_records: Vec<u8>,
    },
    DefineFont {
        character_id: u16,
        data: Vec<u8>,
    },
    DefineFont2 {
        character_id: u16,
        flags: u8,
        name: String,
        num_glyphs: u16,
        data: Vec<u8>,
    },
    DefineFont3 {
        character_id: u16,
        flags: u8,
        name: String,
        num_glyphs: u16,
        data: Vec<u8>,
    },
    DefineFont4 {
        character_id: u16,
        flags: u8,
        name: String,
        data: Vec<u8>,
    },
    SymbolClass {
        symbols: Vec<(u16, String)>,
    },
    DefineBinaryData {
        character_id: u16,
        data: Vec<u8>,
    },
    DefineShape {
        character_id: u16,
        bounds: Rect,
        shapes: Vec<u8>,
    },
    DefineShape2 {
        character_id: u16,
        bounds: Rect,
        shapes: Vec<u8>,
    },
    DefineShape3 {
        character_id: u16,
        bounds: Rect,
        shapes: Vec<u8>,
    },
    DefineShape4 {
        character_id: u16,
        bounds: Rect,
        edge_bounds: Rect,
        flags: u8,
        shapes: Vec<u8>,
    },
    Unknown {
        tag_type: u16,
        data: Vec<u8>,
    },
    End,
}

/// All resources extracted from the SWF
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Resources {
    pub images: HashMap<u16, ImageResource>,
    pub sounds: HashMap<u16, SoundResource>,
    pub sprites: HashMap<u16, SpriteResource>,
    pub scripts: HashMap<u16, ScriptResource>,
    pub texts: HashMap<u16, TextResource>,
    pub fonts: HashMap<u16, FontResource>,
    pub shapes: HashMap<u16, ShapeResource>,
    pub binary_data: HashMap<u16, BinaryDataResource>,
}

/// Image resource (PNG, JPEG, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResource {
    pub id: u16,
    pub width: u16,
    pub height: u16,
    pub format: ImageFormat,
    pub data: Vec<u8>,  // Raw image bytes (PNG/JPEG format)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    JpegWithAlpha,
}

/// Sound resource (MP3, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundResource {
    pub id: u16,
    pub format: SoundFormat,
    pub sample_rate: u32,
    pub stereo: bool,
    pub sample_count: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SoundFormat {
    Uncompressed,
    Adpcm,
    Mp3,
    UncompressedLittleEndian,
    Nellymoser16,
    Nellymoser8,
    Nellymoser,
    Speex,
}

/// Sprite/MovieClip resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteResource {
    pub id: u16,
    pub frame_count: u16,
    pub tags: Vec<Tag>,
}

/// ActionScript resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResource {
    pub id: u16,
    pub name: String,
    pub bytecode: Vec<u8>,
    pub decompiled: Option<String>,
    pub script_type: ScriptType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptType {
    AS1,
    AS2,
    AS3,
}

/// Text resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextResource {
    pub id: u16,
    pub bounds: Rect,
    pub text: String,
    pub raw_data: Vec<u8>,
}

/// Font resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontResource {
    pub id: u16,
    pub name: Option<String>,
    pub num_glyphs: u16,
    pub data: Vec<u8>,
}

/// Shape resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeResource {
    pub id: u16,
    pub bounds: Rect,
    pub edge_bounds: Option<Rect>,
    pub data: Vec<u8>,
}

/// Binary data resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryDataResource {
    pub id: u16,
    pub data: Vec<u8>,
}

/// Information about a resource (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub id: u16,
    pub resource_type: String,
    pub name: Option<String>,
    pub size: usize,
    pub metadata: Option<HashMap<String, String>>,
}
