//! Scene setup and model loading

use bevy::prelude::*;

use crate::types::{AutoRotate, GroundGrid, ModelPath, ModelRoot, OrbitCamera};

/// Setup the 3D scene with model, camera, and lighting
pub fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    model_path: Res<ModelPath>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load the GLTF scene
    let path = model_path.0.clone();
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
        Transform::default(),
        AutoRotate,
        ModelRoot,
    ));

    // Camera with orbit controls (will be repositioned once model loads)
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera::default(),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    // Directional light (sun)
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ground grid
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0).subdivisions(10))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.3, 0.3, 0.3, 0.5),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.001, 0.0), // Slightly below origin to avoid z-fighting
        GroundGrid,
    ));
}

/// Auto-rotate model (currently disabled)
pub fn auto_rotate_model(
    _time: Res<Time>,
    _query: Query<&mut Transform, With<AutoRotate>>,
    _mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    // Auto-rotation disabled - model stays still unless manipulated
}
