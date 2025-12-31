//! UI overlay for view settings

use bevy::pbr::wireframe::{NoWireframe, Wireframe, WireframeColor};
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use crate::types::{GroundGrid, ViewSettings};

// UI component markers
#[derive(Component)]
pub struct SettingsPanel;

#[derive(Component)]
pub struct CheckboxWireframe;

#[derive(Component)]
pub struct CheckboxGrid;

#[derive(Component)]
pub struct CheckboxBones;

#[derive(Component)]
pub struct CheckboxBackground;

#[derive(Component)]
pub struct CheckboxBox;

/// Setup the settings UI panel
pub fn setup_ui(mut commands: Commands) {
    // Root container - positioned top-right
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            BorderRadius::all(Val::Px(8.0)),
            SettingsPanel,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("View Options"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Wireframe checkbox
            spawn_checkbox(parent, "Wireframe (W)", CheckboxWireframe, false);

            // Bones checkbox
            spawn_checkbox(parent, "Skeleton (B)", CheckboxBones, false);

            // Grid checkbox
            spawn_checkbox(parent, "Grid (G)", CheckboxGrid, true);

            // Background checkbox
            spawn_checkbox(parent, "White BG", CheckboxBackground, false);
        });
}

/// Helper to spawn a checkbox row
fn spawn_checkbox<T: Component>(parent: &mut ChildSpawnerCommands, label: &str, marker: T, checked: bool) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            Button,
            marker,
        ))
        .with_children(|row| {
            // Checkbox box
            row.spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BorderRadius::all(Val::Px(3.0)),
                BackgroundColor(if checked {
                    Color::srgb(0.2, 0.6, 1.0)
                } else {
                    Color::NONE
                }),
                CheckboxBox,
            ))
            .with_children(|checkbox| {
                // Checkmark (only visible when checked)
                if checked {
                    checkbox.spawn((
                        Text::new("X"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                }
            });

            // Label
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
            ));
        });
}

/// Handle checkbox button clicks
pub fn handle_checkbox_clicks(
    mut view_settings: ResMut<ViewSettings>,
    interaction_query: Query<
        (
            &Interaction,
            Option<&CheckboxWireframe>,
            Option<&CheckboxGrid>,
            Option<&CheckboxBones>,
            Option<&CheckboxBackground>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, wireframe, grid, bones, background) in &interaction_query {
        if *interaction == Interaction::Pressed {
            if wireframe.is_some() {
                view_settings.show_wireframe = !view_settings.show_wireframe;
            }
            if grid.is_some() {
                view_settings.show_grid = !view_settings.show_grid;
            }
            if bones.is_some() {
                view_settings.show_bones = !view_settings.show_bones;
            }
            if background.is_some() {
                view_settings.white_background = !view_settings.white_background;
            }
        }
    }
}

/// Track wireframe state
#[derive(Resource, Default)]
pub struct WireframeState {
    pub enabled: bool,
    pub original_materials: Vec<(AssetId<StandardMaterial>, Color, AlphaMode)>,
}

/// Sync view settings to actual scene state
pub fn sync_view_settings(
    view_settings: Res<ViewSettings>,
    mut grid_query: Query<&mut Visibility, With<GroundGrid>>,
    mut clear_color: ResMut<ClearColor>,
    mut checkbox_query: Query<(&ChildOf, &mut BackgroundColor, Option<&Children>), With<CheckboxBox>>,
    parent_query: Query<(
        Option<&CheckboxWireframe>,
        Option<&CheckboxGrid>,
        Option<&CheckboxBones>,
        Option<&CheckboxBackground>,
    )>,
    mut commands: Commands,
    text_query: Query<Entity, With<Text>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_query: Query<(Entity, &MeshMaterial3d<StandardMaterial>), Without<GroundGrid>>,
    grid_entity_query: Query<Entity, With<GroundGrid>>,
    no_wireframe_query: Query<(), With<NoWireframe>>,
    mut wireframe_state: ResMut<WireframeState>,
) {
    // Ensure ground grid has NoWireframe
    for entity in &grid_entity_query {
        if no_wireframe_query.get(entity).is_err() {
            commands.entity(entity).insert(NoWireframe);
        }
    }

    // Sync wireframe - only on state change
    if view_settings.show_wireframe != wireframe_state.enabled {
        wireframe_state.enabled = view_settings.show_wireframe;

        if view_settings.show_wireframe {
            // Add per-mesh wireframe with material colors and make materials transparent
            let mut processed = std::collections::HashSet::new();
            for (entity, material_handle) in &mesh_query {
                let id = material_handle.0.id();
                if materials.get(&material_handle.0).is_some() {
                    // Use a visible green color for wireframe (BG3 models use vertex colors, not materials)
                    let wireframe_color = Color::srgb(0.0, 0.9, 0.4);
                    commands.entity(entity).insert((Wireframe, WireframeColor { color: wireframe_color }));

                    if processed.insert(id) {
                        if let Some(mat) = materials.get_mut(id) {
                            wireframe_state.original_materials.push((id, mat.base_color, mat.alpha_mode));
                            mat.alpha_mode = AlphaMode::Blend;
                            mat.base_color = mat.base_color.with_alpha(0.0);
                        }
                    }
                }
            }
        } else {
            // Remove wireframe and restore materials
            for (entity, _) in &mesh_query {
                commands.entity(entity).remove::<(Wireframe, WireframeColor)>();
            }
            for (id, color, alpha_mode) in wireframe_state.original_materials.drain(..) {
                if let Some(mat) = materials.get_mut(id) {
                    mat.base_color = color;
                    mat.alpha_mode = alpha_mode;
                }
            }
        }
    }

    // Sync grid visibility
    for mut visibility in &mut grid_query {
        *visibility = if view_settings.show_grid {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Sync background color
    clear_color.0 = if view_settings.white_background {
        Color::WHITE
    } else {
        Color::srgb(0.1, 0.1, 0.1)
    };

    // Update checkbox visuals
    for (child_of, mut bg_color, children) in &mut checkbox_query {
        if let Ok((wireframe, grid, bones, background)) = parent_query.get(child_of.parent()) {
            let is_checked = if wireframe.is_some() {
                view_settings.show_wireframe
            } else if grid.is_some() {
                view_settings.show_grid
            } else if bones.is_some() {
                view_settings.show_bones
            } else if background.is_some() {
                view_settings.white_background
            } else {
                false
            };

            // Update background color
            bg_color.0 = if is_checked {
                Color::srgb(0.2, 0.6, 1.0)
            } else {
                Color::NONE
            };

            // Handle checkmark visibility
            if let Some(children) = children {
                for child in children.iter() {
                    if text_query.get(child).is_ok() {
                        if is_checked {
                            commands.entity(child).insert(Visibility::Visible);
                        } else {
                            commands.entity(child).insert(Visibility::Hidden);
                        }
                    }
                }
            }
        }
    }
}
