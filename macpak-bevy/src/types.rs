//! Resources and components for the 3D preview

use std::f32::consts::PI;

use bevy::prelude::*;

/// Centralized view settings controlled by UI
#[derive(Resource)]
pub struct ViewSettings {
    pub show_wireframe: bool,
    pub show_grid: bool,
    pub show_bones: bool,
    pub white_background: bool,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            show_wireframe: false,
            show_grid: true,
            show_bones: false,
            white_background: false,
        }
    }
}

/// Path to the model file being previewed
#[derive(Resource)]
pub struct ModelPath(pub String);

/// Marks that we need to fit the camera to the model once it loads
#[derive(Resource, Default)]
pub struct CameraFitPending(pub bool);

/// Stores the computed model bounds for camera reset
#[derive(Resource, Default)]
pub struct ModelBounds {
    pub center: Vec3,
    pub radius: f32,
}

/// Orbit camera component for 3D navigation
#[derive(Component)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub default_radius: f32,
}

impl OrbitCamera {
    pub fn new(focus: Vec3, radius: f32) -> Self {
        Self {
            focus,
            radius,
            yaw: PI, // Front view (180 degrees)
            pitch: 0.3,
            default_radius: radius,
        }
    }

    pub fn reset(&mut self) {
        self.yaw = PI;
        self.pitch = 0.3;
        self.radius = self.default_radius;
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self::new(Vec3::ZERO, 5.0)
    }
}

/// Marker for entities that auto-rotate (currently disabled)
#[derive(Component)]
pub struct AutoRotate;

/// Marker for the ground grid mesh
#[derive(Component)]
pub struct GroundGrid;

/// Marker for the model root entity
#[derive(Component)]
pub struct ModelRoot;
