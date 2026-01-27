//! File preview and selection operations

use std::path::Path;

use floem::prelude::*;

use crate::gui::state::{BrowserState, FileEntry, RawImageData};

/// Maximum preview dimension (width or height) for resizing
/// Note: Kept small to avoid filling vger's texture atlas (each 256x256 RGBA = 256KB)
const MAX_PREVIEW_SIZE: u32 = 256;

pub fn select_file(file: &FileEntry, state: BrowserState) {
    state.preview_name.set(file.name.clone());
    // Clear previous image - increment version to ensure UI updates
    let (version, _) = state.preview_image.get();
    state.preview_image.set((version + 1, None));
    // Clear 3D preview path (will be set if .glb/.gltf selected)
    state.preview_3d_path.set(None);

    if file.is_dir {
        state.preview_info.set("Directory".to_string());
        state.preview_content.set("[Double-click to open]".to_string());
        return;
    }

    state.preview_info.set(format!("{} | {}", file.file_type, file.size_formatted));

    let path = Path::new(&file.path);
    let ext = file.extension.to_lowercase();

    match ext.as_str() {
        "lsx" | "lsj" | "xml" | "txt" | "json" | "lua" => {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let preview = if content.len() > 5000 {
                        format!(
                            "{}...\n\n[Truncated - {} bytes total]",
                            &content[..5000],
                            content.len()
                        )
                    } else {
                        content
                    };
                    state.preview_content.set(preview);
                }
                Err(_) => {
                    state.preview_content.set("[Unable to read file]".to_string());
                }
            }
        }
        "lsf" => {
            state.preview_content.set("[Binary LSF file - double-click to open in editor]".to_string());
        }
        "loca" => {
            // Convert to XML for preview
            let temp_path = std::path::Path::new("/tmp/temp_loca.xml");
            match maclarian::converter::convert_loca_to_xml(path, temp_path) {
                Ok(_) => {
                    match std::fs::read_to_string(temp_path) {
                        Ok(content) => {
                            let preview = if content.len() > 5000 {
                                format!(
                                    "{}...\n\n[Truncated - {} bytes total]",
                                    &content[..5000],
                                    content.len()
                                )
                            } else {
                                content
                            };
                            state.preview_content.set(preview);
                        }
                        Err(_) => {
                            state.preview_content.set("[Unable to read converted file]".to_string());
                        }
                    }
                }
                Err(e) => {
                    state.preview_content.set(format!("[Error converting LOCA: {}]", e));
                }
            }
        }
        "pak" => {
            match maclarian::pak::PakOperations::list(path) {
                Ok(pak_files) => {
                    let preview = format!(
                        "PAK Archive: {} files\n\n{}",
                        pak_files.len(),
                        pak_files.iter().take(50).cloned().collect::<Vec<_>>().join("\n")
                    );
                    state.preview_content.set(preview);
                }
                Err(e) => {
                    state.preview_content.set(format!("[Error reading PAK: {}]", e));
                }
            }
        }
        "dds" => {
            // Load DDS synchronously (decode to raw RGBA, no PNG encoding)
            let current_version = state.preview_image.get().0;
            match load_dds_image(path) {
                Ok(img_data) => {
                    state.preview_content.set(String::new());
                    state.preview_image.set((current_version + 1, Some(img_data)));
                }
                Err(e) => {
                    state.preview_content.set(format!("[Error loading DDS: {}]", e));
                }
            }
        }
        "png" | "jpg" | "jpeg" => {
            // Load image synchronously (decode to raw RGBA, no PNG encoding)
            let current_version = state.preview_image.get().0;
            match load_standard_image(path) {
                Ok(img_data) => {
                    state.preview_content.set(String::new());
                    state.preview_image.set((current_version + 1, Some(img_data)));
                }
                Err(e) => {
                    state.preview_content.set(format!("[Error loading image: {}]", e));
                }
            }
        }
        "glb" | "gltf" => {
            state.preview_content.set("[3D Model - Click button to preview]".to_string());
            state.preview_3d_path.set(Some(file.path.clone()));
        }
        "gr2" => {
            state.preview_content.set("[GR2 Model - Click button to preview]".to_string());
            state.preview_3d_path.set(Some(file.path.clone()));
        }
        "wem" | "wav" => {
            state.preview_content.set("[Audio file]".to_string());
        }
        _ => {
            state.preview_content.set(format!("File type: {}", ext.to_uppercase()));
        }
    }
}

/// Load a DDS file, resize for preview, and return raw RGBA data
fn load_dds_image(path: &Path) -> Result<RawImageData, String> {
    use dds::{ColorFormat, Decoder, ImageViewMut};

    // Open and decode the DDS file
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut decoder = Decoder::new(file).map_err(|e| e.to_string())?;

    // Get dimensions
    let size = decoder.main_size();

    // Create buffer for RGBA8 output (4 bytes per pixel)
    let mut rgba_buffer = vec![0u8; size.pixels() as usize * 4];

    // Create image view and decode
    let view = ImageViewMut::new(&mut rgba_buffer, size, ColorFormat::RGBA_U8)
        .ok_or_else(|| "Failed to create image view".to_string())?;

    decoder.read_surface(view).map_err(|e| format!("Failed to decode DDS: {:?}", e))?;

    // Create image from raw bytes for resizing
    let img: image::RgbaImage = image::ImageBuffer::from_raw(size.width, size.height, rgba_buffer)
        .ok_or_else(|| "Failed to create image buffer".to_string())?;

    // Resize if larger than preview size
    let img = resize_for_preview(img);

    Ok(RawImageData {
        width: img.width(),
        height: img.height(),
        rgba_data: img.into_raw(),
    })
}

/// Load a regular image file (PNG, JPG), resize for preview, and return raw RGBA data
fn load_standard_image(path: &Path) -> Result<RawImageData, String> {
    // Load the image
    let img = image::open(path).map_err(|e| e.to_string())?;
    let img = img.into_rgba8();

    // Resize if larger than preview size
    let img = resize_for_preview(img);

    Ok(RawImageData {
        width: img.width(),
        height: img.height(),
        rgba_data: img.into_raw(),
    })
}

/// Resize an image to fit within the preview pane while maintaining aspect ratio
fn resize_for_preview(img: image::RgbaImage) -> image::RgbaImage {
    use image::imageops::FilterType;

    let (width, height) = (img.width(), img.height());

    // Only resize if larger than max preview size
    if width <= MAX_PREVIEW_SIZE && height <= MAX_PREVIEW_SIZE {
        return img;
    }

    // Calculate new dimensions maintaining aspect ratio
    let scale = if width > height {
        MAX_PREVIEW_SIZE as f32 / width as f32
    } else {
        MAX_PREVIEW_SIZE as f32 / height as f32
    };

    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;

    image::imageops::resize(&img, new_width, new_height, FilterType::Triangle)
}
