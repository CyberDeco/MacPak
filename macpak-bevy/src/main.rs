//! Standalone 3D model preview window using Bevy
//!
//! Usage: macpak-bevy <path-to-glb-file>
//!
//! Controls:
//! - Left mouse drag: Orbit camera
//! - Right mouse drag OR Shift+Left drag: Pan camera
//! - Arrow keys: Pan camera
//! - Scroll wheel / Pinch: Zoom in/out
//! - Two-finger rotate: Orbit camera
//! - R: Reset camera to default view
//! - W: Toggle wireframe mode
//! - G: Toggle ground grid
//! - B: Toggle bone/skeleton visualization

mod bones;
mod camera;
mod scene;
mod types;
mod ui;

use bevy::asset::{AssetPlugin, UnapprovedPathMode};
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use bevy::render::RenderPlugin;
use clap::Parser;

use bones::draw_bones;
use camera::{fit_camera_to_model, handle_keyboard, orbit_camera};
use scene::{auto_rotate_model, setup_scene};
use types::{CameraFitPending, ModelBounds, ModelPath, ViewSettings};
use ui::{handle_checkbox_clicks, setup_ui, sync_view_settings, WireframeState};

#[derive(Parser)]
#[command(name = "macpak-bevy")]
#[command(about = "3D model preview for MacPak")]
struct Args {
    /// Path to the .glb or .gltf file to preview
    file_path: String,
}

fn main() {
    let args = Args::parse();

    // Extract filename for window title
    let file_name = std::path::Path::new(&args.file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "3D Preview".to_string());

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("Preview: {}", file_name),
                        resolution: (800, 600).into(),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ModelPath(args.file_path))
        .insert_resource(CameraFitPending(true))
        .insert_resource(ModelBounds::default())
        .insert_resource(ViewSettings::default())
        .add_plugins(WireframePlugin::default())
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::srgb(0.0, 1.0, 0.0), // Green for testing
        })
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .insert_resource(WireframeState::default())
        .add_systems(Startup, (setup_scene, setup_ui))
        .add_systems(Update, (fit_camera_to_model, orbit_camera))
        .add_systems(Update, (auto_rotate_model, handle_keyboard, draw_bones))
        .add_systems(Update, (handle_checkbox_clicks, sync_view_settings))
        .run();
}
