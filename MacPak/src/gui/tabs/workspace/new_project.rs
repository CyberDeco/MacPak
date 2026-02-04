//! New Project wizard dialog

use floem::prelude::*;
use std::collections::HashMap;

use crate::gui::shared::{ThemeColors, theme_signal};
use crate::gui::state::WorkspaceState;
use crate::gui::utils::{UuidFormat, generate_uuid};
use crate::workspace::Workspace;
use crate::workspace::project::{BuildSettings, ProjectManifest, ProjectMeta};
use crate::workspace::recipe::Recipe;

use maclarian::mods::meta_generator::to_folder_name;

/// New Project dialog overlay (only visible when show_new_dialog is true)
pub fn new_project_dialog(state: WorkspaceState) -> impl IntoView {
    let show = state.show_new_dialog;

    dyn_container(
        move || show.get(),
        move |visible| {
            if visible {
                new_project_form(state.clone()).into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(move |s| {
        if show.get() {
            s.position(floem::style::Position::Absolute)
                .inset(0.0)
                .items_center()
                .justify_center()
                .background(Color::rgba8(0, 0, 0, 150))
                .z_index(100)
        } else {
            s.display(floem::style::Display::None)
        }
    })
}

fn new_project_form(state: WorkspaceState) -> impl IntoView {
    // Form signals
    let mod_name = RwSignal::new(String::new());
    let author = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let selected_recipe_idx = RwSignal::new(0_usize);
    let project_location = RwSignal::new(
        Workspace::default_projects_dir()
            .to_string_lossy()
            .to_string(),
    );
    let form_error = RwSignal::new(String::new());

    let state_for_create = state.clone();
    let state_for_cancel = state.clone();
    let recipes = state.recipes;

    // Dialog card
    v_stack((
        // Title
        label(|| "New Project").style(move |s| {
                let colors = theme_signal()
                    .map(|t| ThemeColors::for_theme(t.get().effective()))
                    .unwrap_or_else(ThemeColors::dark);
                s.font_size(20.0)
                    .font_weight(floem::text::Weight::BOLD)
                    .color(colors.text_primary)
                    .margin_bottom(16.0)
            }),
            // Recipe selector
            label(|| "Recipe (Mod Type)").style(move |s| {
                let colors = theme_signal()
                    .map(|t| ThemeColors::for_theme(t.get().effective()))
                    .unwrap_or_else(ThemeColors::dark);
                s.font_size(13.0)
                    .font_weight(floem::text::Weight::SEMIBOLD)
                    .color(colors.text_secondary)
                    .margin_bottom(4.0)
            }),
            recipe_selector(recipes, selected_recipe_idx),
            // Mod name
            form_field("Mod Name", mod_name),
            // Author
            form_field("Author", author),
            // Description
            form_field("Description", description),
            // Location
            location_field(project_location),
            // Error message
            dyn_container(
                move || form_error.get(),
                move |err| {
                    if err.is_empty() {
                        empty().into_any()
                    } else {
                        label(move || err.clone())
                            .style(move |s| {
                                let colors = theme_signal()
                                    .map(|t| ThemeColors::for_theme(t.get().effective()))
                                    .unwrap_or_else(ThemeColors::dark);
                                s.color(colors.error).font_size(12.0).margin_top(8.0)
                            })
                            .into_any()
                    }
                },
            ),
            // Buttons
            h_stack((
                empty().style(|s| s.flex_grow(1.0)),
                button("Cancel")
                    .action(move || {
                        state_for_cancel.show_new_dialog.set(false);
                    })
                    .style(move |s| {
                        let colors = theme_signal()
                            .map(|t| ThemeColors::for_theme(t.get().effective()))
                            .unwrap_or_else(ThemeColors::dark);
                        s.padding_horiz(20.0)
                            .padding_vert(8.0)
                            .background(colors.bg_elevated)
                            .color(colors.text_primary)
                            .border(1.0)
                            .border_color(colors.border)
                            .border_radius(6.0)
                            .hover(|s| s.background(colors.bg_hover))
                    }),
                button("Create Project")
                    .action(move || {
                        let name = mod_name.get();
                        if name.is_empty() {
                            form_error.set("Mod name is required.".to_string());
                            return;
                        }

                        let all_recipes = recipes.get();
                        let idx = selected_recipe_idx.get();
                        let recipe = match all_recipes.get(idx) {
                            Some(r) => r.clone(),
                            None => {
                                form_error.set("Please select a recipe.".to_string());
                                return;
                            }
                        };

                        let folder = to_folder_name(&name);
                        let uuid = generate_uuid(UuidFormat::Standard);
                        let location = project_location.get();
                        let project_dir = std::path::PathBuf::from(&location).join(&folder);

                        if project_dir.exists() {
                            form_error.set(format!(
                                "Directory already exists: {}",
                                project_dir.display()
                            ));
                            return;
                        }

                        // Build recipe variables with defaults
                        let mut variables = HashMap::new();
                        for var in &recipe.variables {
                            variables.insert(var.name.clone(), var.default.clone());
                        }

                        let manifest = ProjectManifest {
                            project: ProjectMeta {
                                name: name.clone(),
                                folder,
                                author: author.get(),
                                description: description.get(),
                                uuid,
                                version: "1.0.0.0".to_string(),
                                recipe: recipe.recipe.id.clone(),
                            },
                            build: BuildSettings::default(),
                            variables,
                        };

                        match Workspace::create(&project_dir, manifest) {
                            Ok(ws) => {
                                state_for_create.error_message.set(None);
                                state_for_create
                                    .result_message
                                    .set(Some(format!("Created: {}", ws.manifest.project.name)));
                                state_for_create.workspace.set(Some(ws));
                                state_for_create.show_new_dialog.set(false);
                            }
                            Err(e) => {
                                form_error.set(format!("Failed to create project: {}", e));
                            }
                        }
                    })
                    .style(move |s| {
                        let colors = theme_signal()
                            .map(|t| ThemeColors::for_theme(t.get().effective()))
                            .unwrap_or_else(ThemeColors::dark);
                        s.padding_horiz(20.0)
                            .padding_vert(8.0)
                            .background(colors.accent)
                            .color(colors.text_inverse)
                            .border_radius(6.0)
                            .hover(|s| s.background(colors.accent_hover))
                    }),
            ))
            .style(|s| s.width_full().gap(8.0).margin_top(16.0)),
        ))
        .style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.width(500.0)
                .padding(24.0)
                .background(colors.bg_surface)
                .border(1.0)
                .border_color(colors.border)
                .border_radius(8.0)
                .gap(4.0)
        })
}

fn recipe_selector(recipes: RwSignal<Vec<Recipe>>, selected: RwSignal<usize>) -> impl IntoView {
    dyn_container(
        move || recipes.get(),
        move |recipe_list| {
            let buttons: Vec<_> = recipe_list
                .iter()
                .enumerate()
                .map(|(idx, recipe)| {
                    let name = recipe.recipe.name.clone();
                    let desc = recipe.recipe.description.clone();
                    let category = recipe.recipe.category.clone();

                    let item = v_stack((
                        h_stack((
                            label(move || name.clone()).style(move |s| {
                                let colors = theme_signal()
                                    .map(|t| ThemeColors::for_theme(t.get().effective()))
                                    .unwrap_or_else(ThemeColors::dark);
                                s.font_size(13.0)
                                    .font_weight(floem::text::Weight::SEMIBOLD)
                                    .color(colors.text_primary)
                            }),
                            label(move || category.clone()).style(move |s| {
                                let colors = theme_signal()
                                    .map(|t| ThemeColors::for_theme(t.get().effective()))
                                    .unwrap_or_else(ThemeColors::dark);
                                s.font_size(10.0)
                                    .padding_horiz(6.0)
                                    .padding_vert(1.0)
                                    .border_radius(3.0)
                                    .background(colors.bg_hover)
                                    .color(colors.text_muted)
                            }),
                        ))
                        .style(|s| s.gap(6.0).items_center()),
                        label(move || desc.clone()).style(move |s| {
                            let colors = theme_signal()
                                .map(|t| ThemeColors::for_theme(t.get().effective()))
                                .unwrap_or_else(ThemeColors::dark);
                            s.font_size(11.0).color(colors.text_muted)
                        }),
                    ))
                    .style(move |s| {
                        let colors = theme_signal()
                            .map(|t| ThemeColors::for_theme(t.get().effective()))
                            .unwrap_or_else(ThemeColors::dark);
                        let is_selected = selected.get() == idx;
                        let s = s
                            .width_full()
                            .padding(8.0)
                            .border_radius(4.0)
                            .gap(2.0)
                            .cursor(floem::style::CursorStyle::Pointer);
                        if is_selected {
                            s.background(colors.bg_selected)
                                .border(1.0)
                                .border_color(colors.accent)
                        } else {
                            s.background(colors.bg_elevated)
                                .border(1.0)
                                .border_color(Color::TRANSPARENT)
                                .hover(|s| s.background(colors.bg_hover))
                        }
                    })
                    .on_click_stop(move |_| {
                        selected.set(idx);
                    });

                    item
                })
                .collect();

            v_stack_from_iter(buttons)
                .style(|s| s.width_full().gap(4.0).margin_bottom(12.0))
                .into_any()
        },
    )
}

fn form_field(label_text: &'static str, signal: RwSignal<String>) -> impl IntoView {
    v_stack((
        label(move || label_text).style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.font_size(13.0)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .color(colors.text_secondary)
                .margin_top(8.0)
                .margin_bottom(4.0)
        }),
        text_input(signal).style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.width_full()
                .padding(8.0)
                .background(colors.bg_elevated)
                .color(colors.text_primary)
                .border(1.0)
                .border_color(colors.border)
                .border_radius(4.0)
                .font_size(13.0)
        }),
    ))
    .style(|s| s.width_full())
}

fn location_field(location: RwSignal<String>) -> impl IntoView {
    v_stack((
        label(|| "Project Location").style(move |s| {
            let colors = theme_signal()
                .map(|t| ThemeColors::for_theme(t.get().effective()))
                .unwrap_or_else(ThemeColors::dark);
            s.font_size(13.0)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .color(colors.text_secondary)
                .margin_top(8.0)
                .margin_bottom(4.0)
        }),
        h_stack((
            text_input(location).style(move |s| {
                let colors = theme_signal()
                    .map(|t| ThemeColors::for_theme(t.get().effective()))
                    .unwrap_or_else(ThemeColors::dark);
                s.flex_grow(1.0)
                    .flex_basis(0.0)
                    .min_width(0.0)
                    .padding(8.0)
                    .background(colors.bg_elevated)
                    .color(colors.text_primary)
                    .border(1.0)
                    .border_color(colors.border)
                    .border_radius(4.0)
                    .font_size(13.0)
            }),
            button("Choose...")
                .action(move || {
                    let dialog = rfd::FileDialog::new()
                        .set_title("Choose Project Location")
                        .set_directory(Workspace::default_projects_dir());
                    if let Some(path) = dialog.pick_folder() {
                        location.set(path.to_string_lossy().to_string());
                    }
                })
                .style(move |s| {
                    let colors = theme_signal()
                        .map(|t| ThemeColors::for_theme(t.get().effective()))
                        .unwrap_or_else(ThemeColors::dark);
                    s.padding_horiz(12.0)
                        .padding_vert(8.0)
                        .background(colors.bg_elevated)
                        .color(colors.text_primary)
                        .border(1.0)
                        .border_color(colors.border)
                        .border_radius(4.0)
                        .hover(|s| s.background(colors.bg_hover))
                }),
        ))
        .style(|s| s.width_full().gap(8.0)),
    ))
    .style(|s| s.width_full())
}
