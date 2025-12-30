//! Standalone 3D model preview window using Bevy
//!
//! Usage: macpak-bevy <path-to-glb-file>

use std::f32::consts::PI;

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use clap::Parser;

#[derive(Parser)]
#[command(name = "macpak-bevy")]
#[command(about = "3D model preview for MacPak")]
struct Args {
    /// Path to the .glb or .gltf file to preview
    file_path: String,
}

#[derive(Resource)]
struct ModelPath(String);

#[derive(Component)]
struct OrbitCamera {
    focus: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 5.0,
            yaw: 0.0,
            pitch: 0.3,
        }
    }
}

#[derive(Component)]
struct AutoRotate;

fn main() {
    let args = Args::parse();

    // Extract filename for window title
    let file_name = std::path::Path::new(&args.file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "3D Preview".to_string());

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Preview: {}", file_name),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ModelPath(args.file_path))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (orbit_camera, auto_rotate_model))
        .run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>, model_path: Res<ModelPath>) {
    // Load the GLTF scene - clone the path to satisfy lifetime requirements
    let path = model_path.0.clone();
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
        Transform::default(),
        AutoRotate,
    ));

    // Camera with orbit controls
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera::default(),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
    });

    // Directional light (sun)
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn auto_rotate_model(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<AutoRotate>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    // Only auto-rotate when not manually controlling
    if mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.pressed(MouseButton::Right) {
        return;
    }

    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 0.3);
    }
}

fn orbit_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    let mut rotation_delta = Vec2::ZERO;
    let mut zoom_delta: f32 = 0.0;

    // Rotation with left mouse button
    if mouse_buttons.pressed(MouseButton::Left) {
        rotation_delta = accumulated_mouse_motion.delta;
    }

    // Zoom with scroll wheel
    zoom_delta -= accumulated_mouse_scroll.delta.y * 0.5;

    for (mut transform, mut orbit) in &mut query {
        // Apply rotation
        orbit.yaw -= rotation_delta.x * 0.005;
        orbit.pitch -= rotation_delta.y * 0.005;
        orbit.pitch = orbit.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

        // Apply zoom
        orbit.radius = (orbit.radius + zoom_delta).clamp(1.0, 50.0);

        // Calculate new camera position
        let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.radius * orbit.pitch.sin();
        let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

        transform.translation = orbit.focus + Vec3::new(x, y, z);
        transform.look_at(orbit.focus, Vec3::Y);
    }
}
