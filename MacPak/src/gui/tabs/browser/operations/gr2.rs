//! GR2 model conversion operations

use floem::ext_event::create_ext_action;
use floem::prelude::*;
use floem_reactive::Scope;

use crate::gui::state::BrowserState;

use super::directory::load_directory;

/// Result of GR2 conversion
struct GR2ConversionResult {
    success: bool,
    message: String,
}

/// Convert a GR2 file with bundle options
pub fn convert_gr2_file(state: BrowserState, config_state: crate::gui::state::ConfigState) {
    let Some(gr2_path) = state.gr2_convert_path.get() else {
        return;
    };

    // Capture options before closing dialog
    let keep_gr2 = state.gr2_extract_gr2.get();
    let convert_to_glb = state.gr2_convert_to_glb.get();
    let convert_to_gltf = state.gr2_convert_to_gltf.get();
    let extract_textures = state.gr2_extract_textures.get();
    let convert_to_png = state.gr2_convert_to_png.get();
    let game_data = config_state.bg3_data_path.get();

    // Close the dialog
    state.show_gr2_dialog.set(false);
    state.gr2_convert_path.set(None);

    // Check if any conversion is requested
    if !convert_to_glb && !convert_to_gltf && !extract_textures {
        state.status_message.set("No conversion options selected".to_string());
        return;
    }

    // Show loading overlay
    state.is_loading.set(true);
    state.loading_message.set("Converting GR2...".to_string());

    let state_for_callback = state.clone();
    let send = create_ext_action(Scope::new(), move |result: GR2ConversionResult| {
        state_for_callback.is_loading.set(false);
        if result.success {
            state_for_callback.status_message.set(result.message);
            // Refresh the file list to show new files
            if let Some(current) = state_for_callback.current_path.get() {
                load_directory(&current, state_for_callback.clone());
            }
        } else {
            state_for_callback.status_message.set(format!("Error: {}", result.message));
        }
    });

    // Get output directory (same as GR2 file location)
    let output_dir = std::path::Path::new(&gr2_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    rayon::spawn(move || {
        let result = perform_gr2_conversion(
            &gr2_path,
            &output_dir,
            keep_gr2,
            convert_to_glb,
            convert_to_gltf,
            extract_textures,
            convert_to_png,
            &game_data,
        );
        send(result);
    });
}

/// Perform GR2 conversion (runs in background thread)
fn perform_gr2_conversion(
    gr2_path: &str,
    output_dir: &std::path::Path,
    keep_gr2: bool,
    convert_to_glb: bool,
    convert_to_gltf: bool,
    extract_textures: bool,
    convert_to_png: bool,
    game_data_path: &str,
) -> GR2ConversionResult {
    use std::path::Path;

    let gr2_file = Path::new(gr2_path);
    let file_stem = gr2_file.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());

    // Use subfolder if doing more than just GR2->GLB (glTF or textures)
    let use_subfolder = convert_to_gltf || extract_textures;
    let actual_output_dir = if use_subfolder {
        let subfolder = output_dir.join(&file_stem);
        if let Err(e) = std::fs::create_dir_all(&subfolder) {
            return GR2ConversionResult {
                success: false,
                message: format!("Failed to create output folder: {}", e),
            };
        }
        subfolder
    } else {
        output_dir.to_path_buf()
    };

    let mut results = Vec::new();

    // Convert to GLB or glTF
    if convert_to_glb || convert_to_gltf {
        let ext = if convert_to_glb { "glb" } else { "gltf" };
        let output_path = actual_output_dir.join(format!("{}.{}", file_stem, ext));

        let result = if convert_to_glb {
            maclarian::converter::convert_gr2_to_glb(gr2_file, &output_path)
        } else {
            maclarian::converter::convert_gr2_to_gltf(gr2_file, &output_path)
        };

        match result {
            Ok(()) => {
                results.push(format!("{}", ext.to_uppercase()));
            }
            Err(e) => return GR2ConversionResult {
                success: false,
                message: format!("Failed to convert to {}: {}", ext.to_uppercase(), e),
            },
        }
    }

    // Extract textures if requested
    if extract_textures {
        let options = maclarian::gr2_extraction::Gr2ExtractionOptions {
            convert_to_glb: false, // Already converted above if needed
            extract_textures: true,
            extract_virtual_textures: false,
            game_data_path: if game_data_path.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(game_data_path))
            },
            virtual_textures_path: None,
            keep_original_gr2: true,
            convert_to_png,
            // Keep DDS if "Extract textures DDS" is checked (even if also converting to PNG)
            keep_original_dds: true,
        };

        match maclarian::gr2_extraction::process_extracted_gr2_to_dir(gr2_file, &actual_output_dir, &options) {
            Ok(tex_result) => {
                if !tex_result.texture_paths.is_empty() {
                    // Always keep DDS when extract_textures is checked
                    let format = if convert_to_png { "DDS + PNG" } else { "DDS" };
                    results.push(format!("{} {} textures", tex_result.texture_paths.len(), format));
                }
                for warning in tex_result.warnings {
                    eprintln!("Texture extraction warning: {}", warning);
                }
            }
            Err(e) => {
                // Don't fail the whole operation, just report the warning
                eprintln!("Texture extraction failed: {}", e);
            }
        }
    }

    // Copy original GR2 to output directory if requested
    if keep_gr2 && use_subfolder {
        let gr2_dest = actual_output_dir.join(gr2_file.file_name().unwrap_or_default());
        let _ = std::fs::copy(gr2_file, &gr2_dest);
    }

    let folder_info = if use_subfolder {
        format!(" in {}/", file_stem)
    } else {
        String::new()
    };

    if results.is_empty() {
        GR2ConversionResult {
            success: true,
            message: "No output files created".to_string(),
        }
    } else {
        GR2ConversionResult {
            success: true,
            message: format!("Created{}: {}", folder_info, results.join(", ")),
        }
    }
}
