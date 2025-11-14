use crate::core::decompiler;
use crate::core::parser::parse_swf_file;
use crate::core::resources::{extract_sound, to_png};
use crate::core::types::{ImageResource, ResourceInfo, SWFFile, SoundResource};
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

    for (id, script) in &swf.resources.scripts {
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

    Ok(info)
}
