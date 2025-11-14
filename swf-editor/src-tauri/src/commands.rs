use crate::core::decompiler;
use crate::core::parser::parse_swf_file;
use crate::core::types::{ResourceInfo, SWFFile};
use crate::state::SharedState;
use std::collections::HashMap;
use std::fs;
use tauri::State;

/// Open and parse a SWF file
#[tauri::command]
pub async fn open_swf(path: String, state: State<'_, SharedState>) -> Result<SWFFile, String> {
    // Parse the SWF file
    let swf = parse_swf_file(&path).map_err(|e| e.to_string())?;

    // Store in state
    let mut app_state = state.lock().unwrap();
    app_state.set_current_swf(swf.clone());

    Ok(swf)
}

/// Get list of resources of a specific type
#[tauri::command]
pub async fn get_resources(
    resource_type: String,
    state: State<'_, SharedState>,
) -> Result<Vec<ResourceInfo>, String> {
    let app_state = state.lock().unwrap();

    let swf = app_state
        .get_current_swf()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    let mut resources = Vec::new();

    match resource_type.as_str() {
        "images" => {
            for (id, img) in &swf.resources.images {
                let mut metadata = HashMap::new();
                metadata.insert("width".to_string(), img.width.to_string());
                metadata.insert("height".to_string(), img.height.to_string());
                metadata.insert("format".to_string(), format!("{:?}", img.format));

                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "image".to_string(),
                    name: Some(format!("Image_{}", id)),
                    size: img.data.len(),
                    metadata: Some(metadata),
                });
            }
        }
        "sounds" => {
            for (id, sound) in &swf.resources.sounds {
                let mut metadata = HashMap::new();
                metadata.insert("format".to_string(), format!("{:?}", sound.format));
                metadata.insert("sample_rate".to_string(), sound.sample_rate.to_string());
                metadata.insert(
                    "stereo".to_string(),
                    if sound.stereo { "Yes" } else { "No" }.to_string(),
                );

                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "sound".to_string(),
                    name: Some(format!("Sound_{}", id)),
                    size: sound.data.len(),
                    metadata: Some(metadata),
                });
            }
        }
        "sprites" => {
            for (id, sprite) in &swf.resources.sprites {
                let mut metadata = HashMap::new();
                metadata.insert("frame_count".to_string(), sprite.frame_count.to_string());

                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "sprite".to_string(),
                    name: Some(format!("Sprite_{}", id)),
                    size: sprite.tags.len(),
                    metadata: Some(metadata),
                });
            }
        }
        "scripts" => {
            for (id, script) in &swf.resources.scripts {
                let mut metadata = HashMap::new();
                metadata.insert("script_type".to_string(), format!("{:?}", script.script_type));
                metadata.insert("name".to_string(), script.name.clone());

                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "script".to_string(),
                    name: Some(script.name.clone()),
                    size: script.bytecode.len(),
                    metadata: Some(metadata),
                });
            }
        }
        "texts" => {
            for (id, text) in &swf.resources.texts {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "text".to_string(),
                    name: Some(text.text.clone()),
                    size: text.raw_data.len(),
                    metadata: None,
                });
            }
        }
        "fonts" => {
            for (id, font) in &swf.resources.fonts {
                let mut metadata = HashMap::new();
                metadata.insert("num_glyphs".to_string(), font.num_glyphs.to_string());

                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "font".to_string(),
                    name: font.name.clone().or_else(|| Some(format!("Font_{}", id))),
                    size: font.data.len(),
                    metadata: Some(metadata),
                });
            }
        }
        "shapes" => {
            for (id, shape) in &swf.resources.shapes {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "shape".to_string(),
                    name: Some(format!("Shape_{}", id)),
                    size: shape.data.len(),
                    metadata: None,
                });
            }
        }
        "binary_data" => {
            for (id, binary) in &swf.resources.binary_data {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "binary_data".to_string(),
                    name: Some(format!("BinaryData_{}", id)),
                    size: binary.data.len(),
                    metadata: None,
                });
            }
        }
        "all" => {
            // Return all resources
            for (id, img) in &swf.resources.images {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "image".to_string(),
                    name: Some(format!("Image_{}", id)),
                    size: img.data.len(),
                    metadata: None,
                });
            }
            for (id, sound) in &swf.resources.sounds {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "sound".to_string(),
                    name: Some(format!("Sound_{}", id)),
                    size: sound.data.len(),
                    metadata: None,
                });
            }
            for (id, sprite) in &swf.resources.sprites {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "sprite".to_string(),
                    name: Some(format!("Sprite_{}", id)),
                    size: sprite.tags.len(),
                    metadata: None,
                });
            }
            for (id, script) in &swf.resources.scripts {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "script".to_string(),
                    name: Some(script.name.clone()),
                    size: script.bytecode.len(),
                    metadata: None,
                });
            }
            for (id, text) in &swf.resources.texts {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "text".to_string(),
                    name: Some(text.text.clone()),
                    size: text.raw_data.len(),
                    metadata: None,
                });
            }
            for (id, font) in &swf.resources.fonts {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "font".to_string(),
                    name: font.name.clone().or_else(|| Some(format!("Font_{}", id))),
                    size: font.data.len(),
                    metadata: None,
                });
            }
            for (id, shape) in &swf.resources.shapes {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "shape".to_string(),
                    name: Some(format!("Shape_{}", id)),
                    size: shape.data.len(),
                    metadata: None,
                });
            }
            for (id, binary) in &swf.resources.binary_data {
                resources.push(ResourceInfo {
                    id: *id,
                    resource_type: "binary_data".to_string(),
                    name: Some(format!("BinaryData_{}", id)),
                    size: binary.data.len(),
                    metadata: None,
                });
            }
        }
        _ => {
            return Err(format!("Unknown resource type: {}", resource_type));
        }
    }

    Ok(resources)
}

/// Get raw resource data as base64
#[tauri::command]
pub async fn get_resource_data(
    resource_type: String,
    resource_id: u16,
    state: State<'_, SharedState>,
) -> Result<Vec<u8>, String> {
    let app_state = state.lock().unwrap();

    let swf = app_state
        .get_current_swf()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    match resource_type.as_str() {
        "image" => {
            let img = swf
                .resources
                .images
                .get(&resource_id)
                .ok_or_else(|| format!("Image {} not found", resource_id))?;

            Ok(img.data.clone())
        }
        "sound" => {
            let sound = swf
                .resources
                .sounds
                .get(&resource_id)
                .ok_or_else(|| format!("Sound {} not found", resource_id))?;

            Ok(sound.data.clone())
        }
        _ => Err(format!("Unknown resource type: {}", resource_type)),
    }
}

/// Export a resource to a file
#[tauri::command]
pub async fn export_resource(
    resource_type: String,
    resource_id: u16,
    output_path: String,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    let app_state = state.lock().unwrap();

    let swf = app_state
        .get_current_swf()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    match resource_type.as_str() {
        "image" => {
            let img = swf
                .resources
                .images
                .get(&resource_id)
                .ok_or_else(|| format!("Image {} not found", resource_id))?;

            // For now, just write raw data
            fs::write(&output_path, &img.data).map_err(|e| e.to_string())?;
        }
        "sound" => {
            let sound = swf
                .resources
                .sounds
                .get(&resource_id)
                .ok_or_else(|| format!("Sound {} not found", resource_id))?;

            fs::write(&output_path, &sound.data).map_err(|e| e.to_string())?;
        }
        _ => {
            return Err(format!("Unknown resource type: {}", resource_type));
        }
    }

    Ok(())
}

/// Decompile a script
#[tauri::command]
pub async fn decompile_script(
    script_id: u16,
    state: State<'_, SharedState>,
) -> Result<String, String> {
    let app_state = state.lock().unwrap();

    let swf = app_state
        .get_current_swf()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    let script = swf
        .resources
        .scripts
        .get(&script_id)
        .ok_or_else(|| format!("Script {} not found", script_id))?;

    // Check if already decompiled
    if let Some(ref decompiled) = script.decompiled {
        return Ok(decompiled.clone());
    }

    // Decompile
    decompiler::decompile_script(&script.bytecode, script.script_type.clone())
}

/// Export all resources to a directory
#[tauri::command]
pub async fn export_all_resources(
    output_dir: String,
    state: State<'_, SharedState>,
) -> Result<usize, String> {
    let app_state = state.lock().unwrap();

    let swf = app_state
        .get_current_swf()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    // Create output directory
    fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let mut count = 0;

    // Export images
    let images_dir = format!("{}/images", output_dir);
    fs::create_dir_all(&images_dir).map_err(|e| e.to_string())?;

    for (id, img) in &swf.resources.images {
        let path = format!("{}/image_{}.dat", images_dir, id);
        fs::write(&path, &img.data).map_err(|e| e.to_string())?;
        count += 1;
    }

    // Export sounds
    let sounds_dir = format!("{}/sounds", output_dir);
    fs::create_dir_all(&sounds_dir).map_err(|e| e.to_string())?;

    for (id, sound) in &swf.resources.sounds {
        let ext = crate::core::resources::sounds::get_extension(&sound.format);
        let path = format!("{}/sound_{}.{}", sounds_dir, id, ext);
        fs::write(&path, &sound.data).map_err(|e| e.to_string())?;
        count += 1;
    }

    // Export scripts
    let scripts_dir = format!("{}/scripts", output_dir);
    fs::create_dir_all(&scripts_dir).map_err(|e| e.to_string())?;

    for (_id, script) in &swf.resources.scripts {
        let path = format!("{}/{}.txt", scripts_dir, script.name);
        let decompiled = decompiler::decompile_script(&script.bytecode, script.script_type.clone())
            .unwrap_or_else(|e| format!("Error: {}", e));
        fs::write(&path, decompiled).map_err(|e| e.to_string())?;
        count += 1;
    }

    Ok(count)
}

/// Save modified SWF file
#[tauri::command]
pub async fn save_swf(output_path: String, state: State<'_, SharedState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();

    let swf = app_state
        .get_current_swf()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    crate::core::writer::write_swf(swf, &output_path).map_err(|e| e.to_string())
}

/// Get basic SWF file info
#[tauri::command]
pub async fn get_swf_info(state: State<'_, SharedState>) -> Result<HashMap<String, String>, String> {
    let app_state = state.lock().unwrap();

    let swf = app_state
        .get_current_swf()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    let mut info = HashMap::new();
    info.insert("path".to_string(), swf.path.clone());
    info.insert("version".to_string(), swf.header.version.to_string());
    info.insert("signature".to_string(), swf.header.signature.clone());
    info.insert(
        "compressed".to_string(),
        swf.header.compressed.to_string(),
    );
    info.insert(
        "file_length".to_string(),
        swf.header.file_length.to_string(),
    );
    info.insert(
        "frame_rate".to_string(),
        swf.header.frame_rate.to_string(),
    );
    info.insert(
        "frame_count".to_string(),
        swf.header.frame_count.to_string(),
    );
    info.insert("tags_count".to_string(), swf.tags.len().to_string());
    info.insert(
        "images_count".to_string(),
        swf.resources.images.len().to_string(),
    );
    info.insert(
        "sounds_count".to_string(),
        swf.resources.sounds.len().to_string(),
    );
    info.insert(
        "sprites_count".to_string(),
        swf.resources.sprites.len().to_string(),
    );
    info.insert(
        "scripts_count".to_string(),
        swf.resources.scripts.len().to_string(),
    );
    info.insert(
        "texts_count".to_string(),
        swf.resources.texts.len().to_string(),
    );
    info.insert(
        "fonts_count".to_string(),
        swf.resources.fonts.len().to_string(),
    );
    info.insert(
        "shapes_count".to_string(),
        swf.resources.shapes.len().to_string(),
    );
    info.insert(
        "binary_data_count".to_string(),
        swf.resources.binary_data.len().to_string(),
    );

    Ok(info)
}

/// Get debug info about all tags in the SWF
#[tauri::command]
pub async fn get_tags_debug(state: State<'_, SharedState>) -> Result<Vec<String>, String> {
    let app_state = state.lock().unwrap();

    let swf = app_state
        .get_current_swf()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    let mut tags_info = Vec::new();

    for (i, tag) in swf.tags.iter().enumerate() {
        let tag_desc = match tag {
            crate::core::types::Tag::End => "End".to_string(),
            crate::core::types::Tag::SetBackgroundColor { r, g, b } => {
                format!("SetBackgroundColor (RGB: {}, {}, {})", r, g, b)
            }
            crate::core::types::Tag::FileAttributes { .. } => "FileAttributes".to_string(),
            crate::core::types::Tag::DefineBitsLossless { character_id, width, height, .. } => {
                format!("DefineBitsLossless (ID: {}, {}x{})", character_id, width, height)
            }
            crate::core::types::Tag::DefineBitsLossless2 { character_id, width, height, .. } => {
                format!("DefineBitsLossless2 (ID: {}, {}x{})", character_id, width, height)
            }
            crate::core::types::Tag::DefineBitsJPEG { character_id, .. } => {
                format!("DefineBitsJPEG (ID: {})", character_id)
            }
            crate::core::types::Tag::DefineBitsJPEG2 { character_id, .. } => {
                format!("DefineBitsJPEG2 (ID: {})", character_id)
            }
            crate::core::types::Tag::DefineBitsJPEG3 { character_id, .. } => {
                format!("DefineBitsJPEG3 (ID: {})", character_id)
            }
            crate::core::types::Tag::DefineSound { character_id, format, .. } => {
                format!("DefineSound (ID: {}, format: {})", character_id, format)
            }
            crate::core::types::Tag::DefineSprite { character_id, frame_count, .. } => {
                format!("DefineSprite (ID: {}, frames: {})", character_id, frame_count)
            }
            crate::core::types::Tag::DoABC { name, .. } => {
                format!("DoABC ({})", name)
            }
            crate::core::types::Tag::DoAction { actions } => {
                format!("DoAction ({} bytes)", actions.len())
            }
            crate::core::types::Tag::SymbolClass { symbols } => {
                format!("SymbolClass ({} symbols)", symbols.len())
            }
            crate::core::types::Tag::DefineBinaryData { character_id, data } => {
                format!("DefineBinaryData (ID: {}, {} bytes)", character_id, data.len())
            }
            crate::core::types::Tag::Unknown { tag_type, data } => {
                format!("Unknown (type: {}, {} bytes)", tag_type, data.len())
            }
            _ => format!("{:?}", tag).chars().take(50).collect(),
        };

        tags_info.push(format!("#{}: {}", i, tag_desc));
    }

    Ok(tags_info)
}

/// Search SWF for resources matching query
#[tauri::command]
pub async fn search_swf(
    query: String,
    search_mode: String, // "all", "name", "content"
    case_sensitive: bool,
    state: State<'_, SharedState>,
) -> Result<Vec<serde_json::Value>, String> {
    let app_state = state.lock().unwrap();

    let swf_file = app_state
        .current_swf
        .as_ref()
        .ok_or("No SWF file loaded")?;

    let mut results = Vec::new();

    let query_lower = if case_sensitive {
        query.clone()
    } else {
        query.to_lowercase()
    };

    // Helper function to check match
    let matches = |text: &str| -> bool {
        if case_sensitive {
            text.contains(&query)
        } else {
            text.to_lowercase().contains(&query_lower)
        }
    };

    // Search images
    for (_key, resource) in &swf_file.resources.images {
        let name = format!("image_{}", resource.id);
        let should_add = match search_mode.as_str() {
            "name" => matches(&name),
            "content" => false, // Can't search image content
            _ => matches(&name) || matches(&format!("{}", resource.id)),
        };

        if should_add {
            results.push(serde_json::json!({
                "resource_type": "images",
                "resource_id": resource.id,
                "resource_name": name,
                "match_type": "name",
                "preview": format!("{}x{} {} image", resource.width, resource.height, 
                                 match resource.format {
                                     crate::core::types::ImageFormat::Png => "PNG",
                                     crate::core::types::ImageFormat::Jpeg => "JPEG",
                                     crate::core::types::ImageFormat::JpegWithAlpha => "JPEG+Alpha",
                                 }),
            }));
        }
    }

    // Search sounds
    for (_key, resource) in &swf_file.resources.sounds {
        let name = format!("sound_{}", resource.id);
        let should_add = match search_mode.as_str() {
            "name" => matches(&name),
            "content" => false,
            _ => matches(&name) || matches(&format!("{}", resource.id)),
        };

        if should_add {
            results.push(serde_json::json!({
                "resource_type": "sounds",
                "resource_id": resource.id,
                "resource_name": name,
                "match_type": "name",
                "preview": format!("{:?} sound at {} Hz", resource.format, resource.sample_rate),
            }));
        }
    }

    // Search scripts
    for (_key, resource) in &swf_file.resources.scripts {
        let name_match = matches(&resource.name);
        let mut content_match = false;
        let mut preview = String::new();
        let mut line_num = None;

        // Check if we should search content
        if search_mode == "content" || search_mode == "all" {
            // Decompile and search in code
            if let Some(decompiled) = &resource.decompiled {
                if matches(decompiled) {
                    content_match = true;
                    // Find preview snippet
                    if let Some(idx) = if case_sensitive {
                        decompiled.find(&query)
                    } else {
                        decompiled.to_lowercase().find(&query_lower)
                    } {
                        let start = idx.saturating_sub(30);
                        let end = (idx + query.len() + 30).min(decompiled.len());
                        preview = decompiled[start..end].to_string();
                        
                        // Calculate line number
                        line_num = Some(decompiled[..idx].lines().count() + 1);
                    }
                }
            }
        }

        let should_add = match search_mode.as_str() {
            "name" => name_match,
            "content" => content_match,
            _ => name_match || content_match,
        };

        if should_add {
            results.push(serde_json::json!({
                "resource_type": "scripts",
                "resource_id": resource.id,
                "resource_name": resource.name.clone(),
                "match_type": if content_match { "content" } else { "name" },
                "preview": if !preview.is_empty() { preview } else { resource.name.clone() },
                "line_number": line_num,
            }));
        }
    }

    // Search sprites
    for (_key, resource) in &swf_file.resources.sprites {
        let name = format!("sprite_{}", resource.id);
        let should_add = match search_mode.as_str() {
            "name" => matches(&name),
            "content" => false,
            _ => matches(&name) || matches(&format!("{}", resource.id)),
        };

        if should_add {
            results.push(serde_json::json!({
                "resource_type": "sprites",
                "resource_id": resource.id,
                "resource_name": name,
                "match_type": "name",
                "preview": format!("{} frames, {} nested tags", resource.frame_count, resource.tags.len()),
            }));
        }
    }

    // Search texts
    for (_key, resource) in &swf_file.resources.texts {
        let name_match = matches(&resource.text);
        let content_match = matches(&format!("{}", resource.id));

        let should_add = match search_mode.as_str() {
            "name" => name_match,
            "content" => name_match,
            _ => name_match || content_match,
        };

        if should_add {
            results.push(serde_json::json!({
                "resource_type": "texts",
                "resource_id": resource.id,
                "resource_name": resource.text.clone(),
                "match_type": "content",
                "preview": resource.text.clone(),
            }));
        }
    }

    // Search fonts
    for (_key, resource) in &swf_file.resources.fonts {
        let name = resource.name.clone().unwrap_or_else(|| format!("font_{}", resource.id));
        let should_add = match search_mode.as_str() {
            "name" => matches(&name),
            "content" => false,
            _ => matches(&name) || matches(&format!("{}", resource.id)),
        };

        if should_add {
            results.push(serde_json::json!({
                "resource_type": "fonts",
                "resource_id": resource.id,
                "resource_name": name,
                "match_type": "name",
                "preview": format!("{} glyphs", resource.num_glyphs),
            }));
        }
    }

    // Search shapes
    for (_key, resource) in &swf_file.resources.shapes {
        let name = format!("shape_{}", resource.id);
        let should_add = match search_mode.as_str() {
            "name" => matches(&name),
            "content" => false,
            _ => matches(&name) || matches(&format!("{}", resource.id)),
        };

        if should_add {
            results.push(serde_json::json!({
                "resource_type": "shapes",
                "resource_id": resource.id,
                "resource_name": name,
                "match_type": "name",
                "preview": format!("Vector shape ({} bytes)", resource.data.len()),
            }));
        }
    }

    Ok(results)
}

/// Update text resource content
#[tauri::command]
pub async fn update_text(
    text_id: u16,
    new_text: String,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    let mut app_state = state.lock().unwrap();

    let swf = app_state
        .current_swf
        .as_mut()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    // Update text resource
    if let Some(text_resource) = swf.resources.texts.get_mut(&text_id) {
        text_resource.text = new_text;
        Ok(())
    } else {
        Err(format!("Text resource {} not found", text_id))
    }
}

/// Update image resource (replace with new PNG/JPEG data)
#[tauri::command]
pub async fn update_image(
    image_id: u16,
    new_image_data: Vec<u8>,
    width: u16,
    height: u16,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    let mut app_state = state.lock().unwrap();

    let swf = app_state
        .current_swf
        .as_mut()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    // Update image resource
    if let Some(image_resource) = swf.resources.images.get_mut(&image_id) {
        image_resource.data = new_image_data;
        image_resource.width = width;
        image_resource.height = height;
        Ok(())
    } else {
        Err(format!("Image resource {} not found", image_id))
    }
}

/// Update script resource bytecode
#[tauri::command]
pub async fn update_script(
    script_id: u16,
    new_bytecode: Vec<u8>,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    let mut app_state = state.lock().unwrap();

    let swf = app_state
        .current_swf
        .as_mut()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    // Update script resource
    if let Some(script_resource) = swf.resources.scripts.get_mut(&script_id) {
        script_resource.bytecode = new_bytecode;
        script_resource.decompiled = None; // Clear cached decompilation
        Ok(())
    } else {
        Err(format!("Script resource {} not found", script_id))
    }
}

/// Save modified SWF to a new file
#[tauri::command]
pub async fn save_swf_as(
    output_path: String,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    let app_state = state.lock().unwrap();

    let swf = app_state
        .get_current_swf()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    crate::core::writer::write_swf(swf, &output_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn replace_all_text(
    find_text: String,
    replace_text: String,
    case_sensitive: bool,
    state: State<'_, SharedState>,
) -> Result<usize, String> {
    let mut app_state = state.lock().unwrap();
    let swf = app_state.current_swf.as_mut()
        .ok_or_else(|| "No SWF file loaded".to_string())?;

    let mut count = 0;

    // Replace in text resources
    for (_key, text_resource) in swf.resources.texts.iter_mut() {
        let original = text_resource.text.clone();
        if case_sensitive {
            text_resource.text = text_resource.text.replace(&find_text, &replace_text);
        } else {
            // Case-insensitive replacement
            let lower_text = text_resource.text.to_lowercase();
            let lower_find = find_text.to_lowercase();
            if lower_text.contains(&lower_find) {
                let mut result = String::new();
                let mut last_pos = 0;
                for (idx, _) in text_resource.text.match_indices(&find_text) {
                    result.push_str(&text_resource.text[last_pos..idx]);
                    result.push_str(&replace_text);
                    last_pos = idx + find_text.len();
                }
                result.push_str(&text_resource.text[last_pos..]);
                text_resource.text = result;
            }
        }
        if original != text_resource.text {
            count += 1;
        }
    }

    // Note: Script replacement would require recompilation, so we skip it for now
    // Users can manually edit scripts one by one

    Ok(count)
}
