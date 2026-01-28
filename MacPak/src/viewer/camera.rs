//! Camera controls and systems

use std::f32::consts::PI;

use bevy::camera::primitives::MeshAabb;
use bevy::input::gestures::{PinchGesture, RotationGesture};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

use crate::viewer::types::{CameraFitPending, GroundGrid, ModelBounds, OrbitCamera, ViewSettings};

/// Fit camera to model bounds once the model finishes loading
pub fn fit_camera_to_model(
    mut pending: ResMut<CameraFitPending>,
    mut model_bounds: ResMut<ModelBounds>,
    mesh_query: Query<(&GlobalTransform, &Mesh3d), Without<GroundGrid>>,
    meshes: Res<Assets<Mesh>>,
    mut camera_query: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    if !pending.0 {
        return;
    }

    // Collect all mesh AABBs
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    let mut found_any = false;

    for (global_transform, mesh_handle) in &mesh_query {
        if let Some(mesh) = meshes.get(&mesh_handle.0) {
            if let Some(aabb) = mesh.compute_aabb() {
                let center = global_transform.transform_point(aabb.center.into());
                let half_extents: Vec3 = aabb.half_extents.into();

                // Transform AABB corners to world space (approximate)
                let world_half = half_extents * global_transform.compute_transform().scale;

                min = min.min(center - world_half);
                max = max.max(center + world_half);
                found_any = true;
            }
        }
    }

    if !found_any {
        return; // Model not loaded yet
    }

    // Calculate bounds
    let center = (min + max) / 2.0;
    let size = max - min;
    let radius = size.length() / 2.0;

    // Store for reset functionality
    model_bounds.center = center;
    model_bounds.radius = radius.max(0.5); // Minimum radius

    // Update camera
    let camera_distance = radius * 2.5; // Distance to fit model nicely

    for (mut transform, mut orbit) in &mut camera_query {
        orbit.focus = center;
        orbit.radius = camera_distance;
        orbit.default_radius = camera_distance;

        // Update transform immediately
        let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.radius * orbit.pitch.sin() + center.y;
        let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

        transform.translation = orbit.focus + Vec3::new(x, y - center.y, z);
        transform.look_at(orbit.focus, Vec3::Y);
    }

    pending.0 = false;
}

/// Handle orbit camera controls (mouse, trackpad, gestures)
pub fn orbit_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pinch_events: MessageReader<PinchGesture>,
    mut rotate_events: MessageReader<RotationGesture>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
    model_bounds: Res<ModelBounds>,
) {
    let mut rotation_delta = Vec2::ZERO;
    let mut pan_delta = Vec2::ZERO;
    let mut zoom_delta: f32 = 0.0;

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let left_pressed = mouse_buttons.pressed(MouseButton::Left);
    let right_pressed = mouse_buttons.pressed(MouseButton::Right);

    // Pan with right mouse OR Shift+Left (for trackpads)
    if right_pressed || (shift_held && left_pressed) {
        pan_delta = accumulated_mouse_motion.delta;
    }
    // Rotation with left mouse button (only if not panning)
    else if left_pressed {
        rotation_delta = accumulated_mouse_motion.delta;
    }

    // Zoom with scroll wheel
    zoom_delta -= accumulated_mouse_scroll.delta.y * 0.05;

    // Pinch to zoom (trackpad)
    for pinch in pinch_events.read() {
        zoom_delta -= pinch.0 * 0.5;
    }

    // Two-finger rotate (trackpad)
    for rotate in rotate_events.read() {
        rotation_delta.x += rotate.0 * 15.0;
    }

    for (mut transform, mut orbit) in &mut query {
        // Apply rotation
        orbit.yaw -= rotation_delta.x * 0.002;
        orbit.pitch -= rotation_delta.y * 0.002;
        orbit.pitch = orbit.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

        // Apply pan (move focus point)
        if pan_delta != Vec2::ZERO {
            // Calculate camera right and up vectors
            let forward = (orbit.focus - transform.translation).normalize();
            let right = forward.cross(Vec3::Y).normalize();
            let up = right.cross(forward).normalize();

            // Scale pan speed with distance
            let pan_speed = orbit.radius * 0.002;
            orbit.focus += right * (-pan_delta.x * pan_speed);
            orbit.focus += up * (pan_delta.y * pan_speed);
        }

        // Apply zoom with bounds-aware limits
        let min_radius = model_bounds.radius.max(0.1) * 0.5;
        let max_radius = model_bounds.radius.max(1.0) * 10.0;
        orbit.radius = (orbit.radius + zoom_delta).clamp(min_radius, max_radius);

        // Calculate new camera position
        let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.radius * orbit.pitch.sin();
        let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

        transform.translation = orbit.focus + Vec3::new(x, y, z);
        transform.look_at(orbit.focus, Vec3::Y);
    }
}

/// Handle keyboard input for camera and view settings
pub fn handle_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut Transform, &mut OrbitCamera)>,
    mut view_settings: ResMut<ViewSettings>,
    model_bounds: Res<ModelBounds>,
) {
    // R - Reset camera
    if keyboard.just_pressed(KeyCode::KeyR) {
        for (mut transform, mut orbit) in &mut camera_query {
            orbit.focus = model_bounds.center;
            orbit.reset();

            let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
            let y = orbit.radius * orbit.pitch.sin();
            let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

            transform.translation = orbit.focus + Vec3::new(x, y, z);
            transform.look_at(orbit.focus, Vec3::Y);
        }
    }

    // W - Toggle wireframe
    if keyboard.just_pressed(KeyCode::KeyW) {
        view_settings.show_wireframe = !view_settings.show_wireframe;
    }

    // G - Toggle grid
    if keyboard.just_pressed(KeyCode::KeyG) {
        view_settings.show_grid = !view_settings.show_grid;
    }

    // B - Toggle bones
    if keyboard.just_pressed(KeyCode::KeyB) {
        view_settings.show_bones = !view_settings.show_bones;
    }

    // Arrow keys - Pan camera
    let mut pan = Vec2::ZERO;
    if keyboard.pressed(KeyCode::ArrowLeft) {
        pan.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        pan.x += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        pan.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        pan.y -= 1.0;
    }

    if pan != Vec2::ZERO {
        for (mut transform, mut orbit) in &mut camera_query {
            let forward = (orbit.focus - transform.translation).normalize();
            let right = forward.cross(Vec3::Y).normalize();
            let up = right.cross(forward).normalize();

            let pan_speed = orbit.radius * 0.005;
            orbit.focus += right * (pan.x * pan_speed);
            orbit.focus += up * (pan.y * pan_speed);

            // Update camera position
            let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
            let y = orbit.radius * orbit.pitch.sin();
            let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

            transform.translation = orbit.focus + Vec3::new(x, y, z);
            transform.look_at(orbit.focus, Vec3::Y);
        }
    }
}
