# SWF Editor - Adobe Flash Decompiler & Editor

A modern, high-performance application for decompiling, viewing, and editing SWF (Adobe Flash) files. Built with Rust and Tauri for maximum performance, and React for a clean, intuitive UI.

## Features

### Core Functionality
- ✅ **Parse SWF Files**: Support for FWS, CWS (ZLIB), and ZWS (LZMA) compressed formats
- ✅ **Resource Extraction**: Extract images, sounds, sprites, scripts, fonts, and more
- ✅ **ActionScript Decompilation**: Basic decompilation of AS1/AS2/AS3 bytecode
- ✅ **Drag & Drop Interface**: Simply drag a .swf file into the window
- ✅ **Resource Preview**: View resources directly in the application
- ✅ **Export Resources**: Export individual resources or all at once

### Supported Resource Types
- 🖼️ **Images**: DefineBitsLossless, DefineBitsJPEG (PNG/JPEG formats)
- 🔊 **Sounds**: DefineSound (MP3, ADPCM, uncompressed)
- 🎬 **Sprites**: DefineSprite (MovieClips with frames)
- ⚡ **Scripts**: DoABC (AS3), DoAction (AS1/AS2)
- 📝 **Text**: DefineText
- 🔤 **Fonts**: DefineFont
- 🎨 **Shapes**: DefineShape (vector graphics)
- 📦 **Binary Data**: DefineBinaryData

## Architecture

```
swf-editor/
├── src-tauri/              # Rust backend
│   ├── core/
│   │   ├── types.rs       # Data structures
│   │   ├── parser/        # SWF parser
│   │   │   ├── header.rs
│   │   │   ├── tags.rs
│   │   │   └── compression.rs
│   │   ├── decompiler/    # ActionScript decompiler
│   │   ├── resources/     # Resource extractors
│   │   └── writer/        # SWF writer (TODO)
│   ├── commands.rs        # Tauri commands
│   ├── state.rs          # State management
│   └── lib.rs
├── src/                   # React frontend
│   ├── components/
│   │   ├── FileTree.tsx   # Resource tree view
│   │   └── Viewer.tsx     # Resource preview
│   ├── types.ts          # TypeScript types
│   └── App.tsx
└── package.json
```

## Tech Stack

### Backend (Rust)
- **Tauri 2**: Application framework
- **flate2**: ZLIB decompression
- **lzma-rs**: LZMA decompression
- **byteorder**: Binary data reading
- **image**: Image processing
- **nom**: Parser combinators
- **rayon**: Parallel processing
- **serde**: Serialization

### Frontend (React + TypeScript)
- **React 19**: UI framework
- **TypeScript**: Type safety
- **Vite**: Build tool
- **Tauri plugins**: File system, dialogs

## Building from Source

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) 18+
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/) for your OS

### Development

```bash
# Install dependencies
npm install

# Run in development mode (requires system dependencies on Linux)
npm run tauri dev
```

### Production Build (Windows)

```bash
# Build for production
npm run tauri build
```

The executable will be in `src-tauri/target/release/`.

## Usage

### Opening SWF Files
1. **Drag & Drop**: Drag a .swf file into the window
2. **File Menu**: Click "Open SWF" and select a file

### Viewing Resources
1. Open a SWF file
2. Browse the resource tree on the left
3. Click on any resource to preview it
4. View metadata and properties in the right panel

### Exporting Resources
- **Single Resource**: Click "Export" button in the viewer
- **All Resources**: Click "Export All" in the toolbar

## SWF Format Support

### Compression Formats
- ✅ **FWS**: Uncompressed
- ✅ **CWS**: ZLIB compressed
- ✅ **ZWS**: LZMA compressed

### Supported Tags
| Tag ID | Name | Status |
|--------|------|--------|
| 0 | End | ✅ |
| 9 | SetBackgroundColor | ✅ |
| 12 | DoAction (AS1/2) | ✅ |
| 14 | DefineSound | ✅ |
| 20 | DefineBitsLossless | ✅ |
| 21 | DefineBitsJPEG | ✅ |
| 35 | DefineBitsJPEG2 | ✅ |
| 36 | DefineBitsLossless2 | ✅ |
| 39 | DefineSprite | ✅ |
| 76 | SymbolClass | ✅ |
| 82 | DoABC (AS3) | ✅ |
| 87 | DefineBinaryData | ✅ |
| 90 | DefineBitsJPEG3 | ✅ |

## Development Roadmap

### Phase 1 - Parser (✅ COMPLETED)
- [x] SWF header parsing
- [x] ZLIB/LZMA decompression
- [x] Basic tag parsing
- [x] Resource extraction

### Phase 2 - UI (✅ COMPLETED)
- [x] Drag & drop interface
- [x] File tree viewer
- [x] Resource preview
- [x] File info display

### Phase 3 - Decompilation (📋 NEXT)
- [x] Basic AS3 hex dump
- [ ] Full AS3 decompiler
- [ ] AS1/AS2 decompiler
- [ ] Syntax highlighting

### Phase 4 - Editing (📋 TODO)
- [ ] Replace images
- [ ] Edit texts
- [ ] Delete resources
- [ ] Save modified SWF

### Phase 5 - Advanced Features (📋 TODO)
- [ ] Search across resources
- [ ] Compare two SWF files
- [ ] Batch processing
- [ ] Shape to SVG conversion

## License

MIT License

---

**Made for Dofus asset modding**
