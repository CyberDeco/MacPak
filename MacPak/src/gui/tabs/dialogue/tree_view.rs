//! Dialog tree view panel

use floem::event::EventPropagation;
use floem::prelude::*;
use floem::text::{Attrs, AttrsList, Style as FontStyle, TextLayout, Weight, Wrap};
use floem::views::{clip, rich_text, virtual_list, VirtualDirection, VirtualItemSize};
use im::Vector as ImVector;
use MacLarian::dialog::NodeConstructor;
use crate::gui::state::{DialogueState, DisplayNode};
use super::context_menu::show_node_context_menu;

const NODE_ROW_HEIGHT: f64 = 32.0;

/// A styled span for rich text rendering
struct StyledSpan {
    start: usize,
    end: usize,
    italic: bool,
    bold: bool,
}

/// Parse HTML-like tags and return plain text with style spans
fn parse_html_styles(text: &str) -> (String, Vec<StyledSpan>) {
    let mut plain_text = String::new();
    let mut spans = Vec::new();
    let mut italic_stack: Vec<usize> = Vec::new();
    let mut bold_stack: Vec<usize> = Vec::new();

    let mut i = 0;
    let chars: Vec<char> = text.chars().collect();

    while i < chars.len() {
        if chars[i] == '<' {
            // Check for <i> tag
            if i + 2 < chars.len() && chars[i + 1] == 'i' && chars[i + 2] == '>' {
                italic_stack.push(plain_text.len());
                i += 3;
                continue;
            }
            // Check for </i> tag
            if i + 3 < chars.len() && chars[i + 1] == '/' && chars[i + 2] == 'i' && chars[i + 3] == '>' {
                if let Some(start) = italic_stack.pop() {
                    spans.push(StyledSpan {
                        start,
                        end: plain_text.len(),
                        italic: true,
                        bold: false,
                    });
                }
                i += 4;
                continue;
            }
            // Check for <b> tag
            if i + 2 < chars.len() && chars[i + 1] == 'b' && chars[i + 2] == '>' {
                bold_stack.push(plain_text.len());
                i += 3;
                continue;
            }
            // Check for </b> tag
            if i + 3 < chars.len() && chars[i + 1] == '/' && chars[i + 2] == 'b' && chars[i + 3] == '>' {
                if let Some(start) = bold_stack.pop() {
                    spans.push(StyledSpan {
                        start,
                        end: plain_text.len(),
                        italic: false,
                        bold: true,
                    });
                }
                i += 4;
                continue;
            }
            // Check for <br> or <br/> tag - convert to space
            if i + 3 < chars.len() && chars[i + 1] == 'b' && chars[i + 2] == 'r' {
                if chars[i + 3] == '>' {
                    plain_text.push(' ');
                    i += 4;
                    continue;
                } else if i + 4 < chars.len() && chars[i + 3] == '/' && chars[i + 4] == '>' {
                    plain_text.push(' ');
                    i += 5;
                    continue;
                }
            }
        }
        plain_text.push(chars[i]);
        i += 1;
    }

    (plain_text, spans)
}

/// Create a TextLayout with HTML styling applied
fn create_styled_text_layout(text: &str, font_size: f32, text_color: floem::peniko::Color) -> TextLayout {
    let (plain_text, spans) = parse_html_styles(text);

    // Create attrs list with base styling
    let mut attrs_list = AttrsList::new(
        Attrs::new()
            .font_size(font_size)
            .color(text_color)
    );

    // Add italic spans
    for span in spans {
        if span.italic && span.start < span.end {
            attrs_list.add_span(
                span.start..span.end,
                Attrs::new()
                    .font_size(font_size)
                    .style(FontStyle::Italic)
                    .color(text_color)
            );
        }
        if span.bold && span.start < span.end {
            attrs_list.add_span(
                span.start..span.end,
                Attrs::new()
                    .font_size(font_size)
                    .weight(Weight::BOLD)
                    .color(text_color)
            );
        }
    }

    let mut layout = TextLayout::new();
    layout.set_text(&plain_text, attrs_list);
    layout
}

/// Panel showing the dialog tree structure
pub fn tree_view_panel(state: DialogueState) -> impl IntoView {
    let state_for_header = state.clone();
    let state_for_list = state.clone();

    v_stack((
        // Dialog header with info
        dialog_header(state_for_header),

        // Node tree
        node_tree(state_for_list),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .min_height(0.0)  // Critical for scroll to work
            .background(Color::WHITE)
    })
}

/// Header showing dialog info
fn dialog_header(state: DialogueState) -> impl IntoView {
    let dialog = state.current_dialog;

    dyn_container(
        move || dialog.get(),
        move |d| {
            if let Some(dialog) = d {
                let node_count = dialog.node_count();
                let root_count = dialog.root_nodes.len();
                let synopsis = dialog.editor_data.synopsis.clone()
                    .unwrap_or_else(|| "No synopsis".to_string());

                v_stack((
                    h_stack((
                        label(move || format!("{} nodes", node_count))
                            .style(|s| {
                                s.font_size(12.0)
                                    .color(Color::rgb8(100, 100, 100))
                                    .padding_horiz(8.0)
                                    .padding_vert(2.0)
                                    .background(Color::rgb8(240, 240, 240))
                                    .border_radius(4.0)
                            }),
                        label(move || format!("{} roots", root_count))
                            .style(|s| {
                                s.font_size(12.0)
                                    .color(Color::rgb8(100, 100, 100))
                                    .padding_horiz(8.0)
                                    .padding_vert(2.0)
                                    .background(Color::rgb8(240, 240, 240))
                                    .border_radius(4.0)
                            }),
                    ))
                    .style(|s| s.gap(8.0)),

                    rich_text(move || {
                        let mut layout = TextLayout::new();
                        let attrs = Attrs::new()
                            .font_size(12.0)
                            .color(floem::peniko::Color::rgba8(80, 80, 80, 255));
                        layout.set_text(&synopsis, AttrsList::new(attrs));
                        layout.set_wrap(Wrap::Word);
                        layout
                    }),
                ))
                .style(|s| s.gap(4.0).width_full())
                .into_any()
            } else {
                empty().into_any()
            }
        },
    )
    .style(|s| {
        s.width_full()
            .padding(12.0)
            .border_bottom(1.0)
            .border_color(Color::rgb8(230, 230, 230))
    })
}

/// The scrollable tree of nodes using virtual_list for performance
fn node_tree(state: DialogueState) -> impl IntoView {
    let display_nodes = state.display_nodes;
    let selected_uuid = state.selected_node_uuid;
    let max_content_width = state.max_content_width;
    let tree_version = state.tree_version;

    // Cache the filtered results to avoid returning a new collection on every call
    use std::rc::Rc;
    use std::cell::RefCell;
    let cached_result: Rc<RefCell<(u64, usize, ImVector<DisplayNode>)>> =
        Rc::new(RefCell::new((u64::MAX, 0, ImVector::new())));
    let cache = cached_result.clone();

    clip(
        scroll(
            virtual_list(
                VirtualDirection::Vertical,
                VirtualItemSize::Fixed(Box::new(|| NODE_ROW_HEIGHT)),
                move || {
                    let version = tree_version.get();
                    let all_nodes = display_nodes.get();
                    let total_count = all_nodes.len();

                    // Check if we need to recompute
                    let mut cache_ref = cache.borrow_mut();
                    let (cached_version, cached_total, cached_im) = &mut *cache_ref;

                    // Recompute if version changed OR total nodes changed (new dialog loaded)
                    if *cached_version != version || *cached_total != total_count {
                        // Filter to only visible nodes - don't rely on CSS to hide
                        // This ensures virtual_list scroll math is correct
                        let filtered: ImVector<_> = all_nodes.into_iter()
                            .filter(|node| node.is_visible.get_untracked())
                            .collect();

                        *cached_version = version;
                        *cached_total = total_count;
                        *cached_im = filtered;
                    }

                    cached_im.clone()
                },
                // Use only UUID as key - stable across expand/collapse
                |node| {
                    node.uuid.clone()
                },
                {
                    let state_for_row = state.clone();
                    move |node| {
                        node_row(node, selected_uuid, display_nodes, tree_version, max_content_width.get(), state_for_row.clone())
                    }
                },
            )
            .style(|s| s.flex_col())
        )
        .style(|s| {
            s.width_full()
                .height_full()
        })
    )
    .style(|s| {
        s.width_full()
            .height_full()
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)
    })
}

/// Update visibility of all descendants when a node is expanded/collapsed
/// Uses untracked read to avoid creating reactive subscriptions in click handlers
fn update_descendant_visibility(
    parent_uuid: &str,
    parent_expanded: bool,
    display_nodes: RwSignal<Vec<DisplayNode>>,
) {
    // Use with_untracked to avoid creating reactive subscriptions
    // This prevents other panels from re-rendering when we expand/collapse nodes
    display_nodes.with_untracked(|nodes| {
        // When expanding: direct children become visible
        // When collapsing: all descendants become invisible
        for node in nodes.iter() {
            if node.parent_uuid.as_deref() == Some(parent_uuid) {
                // Direct children: visible if parent is expanded
                node.is_visible.set(parent_expanded);

                // Recursively update all descendants
                if node.child_count > 0 {
                    let node_is_expanded = node.is_expanded.get_untracked();
                    let descendants_visible = parent_expanded && node_is_expanded;
                    update_descendants_recursive(&node.uuid, descendants_visible, nodes);
                }
            }
        }
    });
}

/// Recursively update visibility of descendants
fn update_descendants_recursive(parent_uuid: &str, parent_visible: bool, all_nodes: &[DisplayNode]) {
    for node in all_nodes.iter() {
        if node.parent_uuid.as_deref() == Some(parent_uuid) {
            node.is_visible.set(parent_visible);

            // Continue recursion - children are visible only if this node is also expanded
            if node.child_count > 0 {
                let node_is_expanded = node.is_expanded.get_untracked();
                let child_visible = parent_visible && node_is_expanded;
                update_descendants_recursive(&node.uuid, child_visible, all_nodes);
            }
        }
    }
}

/// Single node row in the tree
fn node_row(
    node: DisplayNode,
    selected_uuid: RwSignal<Option<String>>,
    display_nodes: RwSignal<Vec<DisplayNode>>,
    tree_version: RwSignal<u64>,
    max_content_width: f32,
    state: DialogueState,
) -> impl IntoView {
    let constructor = node.constructor.clone();
    let text = node.text.clone();
    let speaker = node.speaker_name.clone();
    let depth = node.depth;
    let is_end = node.is_end_node;
    let child_count = node.child_count;
    let is_expanded = node.is_expanded;
    let node_uuid = node.uuid.clone();
    let node_uuid_for_select = node.uuid.clone();
    let node_uuid_for_style = node.uuid.clone();
    let node_for_ctx = node.clone();
    let roll_success = node.roll_success;
    let constructor_for_roll = node.constructor.clone();
    // Get NodeContext (the primary dev note field) if available
    let node_context = node.editor_data.get("NodeContext")
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_default();
    let has_dev_notes = !node_context.is_empty();

    // Format check flags for display
    let check_flags_str = if node.check_flags.is_empty() {
        String::new()
    } else {
        node.check_flags.iter()
            .map(|f| {
                if f.value {
                    f.name.clone()
                } else {
                    format!("{} = False", f.name)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let has_check_flags = !check_flags_str.is_empty();

    h_stack((
        // Indentation
        empty().style(move |s| s.width((depth * 20) as f32)),

        // Expand/collapse indicator (clickable)
        {
            let has_children = child_count > 0;
            let uuid_for_click = node_uuid.clone();
            if has_children {
                label(move || if is_expanded.get() { "▼" } else { "▶" })
                    .style(|s| {
                        s.font_size(10.0)
                            .width(16.0)
                            .color(Color::rgb8(120, 120, 120))
                            .cursor(floem::style::CursorStyle::Pointer)
                    })
                    .on_click_stop(move |_| {
                        let new_expanded = !is_expanded.get();
                        is_expanded.set(new_expanded);
                        update_descendant_visibility(&uuid_for_click, new_expanded, display_nodes);
                        tree_version.update(|v| *v += 1);
                    })
                    .into_any()
            } else {
                empty().style(|s| s.width(16.0)).into_any()
            }
        },

        // Node type badge
        node_type_badge(constructor),

        // Speaker (if present)
        {
            let speaker_check = speaker.clone();
            let speaker_display = speaker.clone();
            dyn_container(
                move || !speaker_check.is_empty(),
                move |has_speaker| {
                    let speaker_inner = speaker_display.clone();
                    if has_speaker {
                        label(move || format!("[{}]", speaker_inner.clone()))
                            .style(|s| {
                                s.font_size(12.0)
                                    .color(Color::rgb8(79, 70, 229))
                                    .font_weight(Weight::MEDIUM)
                                    .margin_right(4.0)
                            })
                            .into_any()
                    } else {
                        empty().into_any()
                    }
                },
            )
        },

        // Check flags (conditions) - show inline before the text
        {
            let flags_display = check_flags_str.clone();
            dyn_container(
                move || has_check_flags,
                move |show_flags| {
                    let flags_inner = flags_display.clone();
                    if show_flags {
                        label(move || format!("[{}]", flags_inner.clone()))
                            .style(|s| {
                                s.font_size(11.0)
                                    .color(Color::rgb8(180, 100, 60)) // Orange-brown for conditions
                                    .font_style(floem::text::Style::Italic)
                                    .margin_right(4.0)
                            })
                            .into_any()
                    } else {
                        empty().into_any()
                    }
                },
            )
        },

        // Node text with HTML styling support (no truncation - use horizontal scroll)
        {
            let text_for_display = text.clone();
            let text_is_empty = text.is_empty();
            let text_len = text.len();
            rich_text(move || {
                let display_text = if text_for_display.is_empty() {
                    "(no text)".to_string()
                } else {
                    text_for_display.clone()
                };

                let text_color = if text_is_empty {
                    floem::peniko::Color::rgba8(180, 180, 180, 255)
                } else {
                    floem::peniko::Color::rgba8(40, 40, 40, 255)
                };

                create_styled_text_layout(&display_text, 13.0, text_color)
            })
            // Set minimum width based on text length (~7px per char at 13pt font)
            // This communicates the content width to the layout system
            .style(move |s| s.min_width((text_len as f32 * 7.0).max(50.0)).flex_shrink(0.0))
        },

        // End node indicator
        dyn_container(
            move || is_end,
            move |is_end| {
                if is_end {
                    label(|| "END")
                        .style(|s| {
                            s.font_size(10.0)
                                .padding_horiz(4.0)
                                .padding_vert(1.0)
                                .background(Color::rgb8(239, 68, 68))
                                .color(Color::WHITE)
                                .border_radius(2.0)
                        })
                        .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),

        // Roll success/failure indicator for RollResult nodes
        dyn_container(
            move || constructor_for_roll == NodeConstructor::RollResult && roll_success.is_some(),
            move |show_indicator| {
                if show_indicator {
                    let is_success = roll_success.unwrap_or(false);
                    let (indicator, bg_color) = if is_success {
                        ("✓", Color::rgb8(34, 197, 94))  // Green for success
                    } else {
                        ("✗", Color::rgb8(239, 68, 68))  // Red for failure
                    };
                    label(move || indicator)
                        .style(move |s| {
                            s.font_size(10.0)
                                .font_weight(Weight::BOLD)
                                .padding_horiz(4.0)
                                .padding_vert(1.0)
                                .background(bg_color)
                                .color(Color::WHITE)
                                .border_radius(2.0)
                        })
                        .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),

        // Dev notes indicator - shows NodeContext when available
        {
            let notes = node_context.clone();
            dyn_container(
                move || has_dev_notes,
                move |show_notes| {
                    let notes_inner = notes.clone();
                    if show_notes {
                        label(move || format!("📝 {}", notes_inner.clone()))
                            .style(|s| {
                                s.font_size(10.0)
                                    .color(Color::rgb8(100, 100, 100))
                                    .font_style(floem::text::Style::Italic)
                                    .max_width(500.0)
                            })
                            .into_any()
                    } else {
                        empty().into_any()
                    }
                },
            )
        },
    ))
    .on_click_stop(move |_| {
        // All rendered rows are visible (filtered at data source level)
        selected_uuid.set(Some(node_uuid_for_select.clone()));
    })
    .on_secondary_click(move |_| {
        // Right-click: show context menu
        show_node_context_menu(&node_for_ctx, state.clone());
        EventPropagation::Stop
    })
    .style(move |s| {
        // Visibility is handled by filtering at virtual_list data source level
        // No need for CSS hiding which caused scroll position bugs
        let is_selected = selected_uuid.get().as_ref() == Some(&node_uuid_for_style);

        // Use max_content_width so all rows are same width for proper scrolling
        let base = s
            .min_width(max_content_width)
            .height(NODE_ROW_HEIGHT)
            .padding_horiz(8.0)
            .padding_right(24.0)
            .gap(4.0)
            .items_center()
            .border_bottom(1.0)
            .border_color(Color::rgb8(245, 245, 245))
            .cursor(floem::style::CursorStyle::Pointer);

        if is_selected {
            base.background(Color::rgb8(227, 242, 253))
        } else {
            base.background(Color::WHITE)
                .hover(|s| s.background(Color::rgb8(250, 250, 250)))
        }
    })
}

/// Badge showing node type with appropriate color
fn node_type_badge(constructor: NodeConstructor) -> impl IntoView {
    let (text, bg_color) = match constructor {
        NodeConstructor::TagQuestion => ("Q", Color::rgb8(59, 130, 246)),
        NodeConstructor::TagAnswer => ("A", Color::rgb8(34, 197, 94)),
        NodeConstructor::ActiveRoll => ("R", Color::rgb8(249, 115, 22)),
        NodeConstructor::PassiveRoll => ("PR", Color::rgb8(249, 115, 22)),
        NodeConstructor::RollResult => ("RR", Color::rgb8(168, 85, 247)),
        NodeConstructor::Alias => ("@", Color::rgb8(156, 163, 175)),
        NodeConstructor::Jump => ("J", Color::rgb8(236, 72, 153)),
        NodeConstructor::Pop => ("P", Color::rgb8(107, 114, 128)),
        NodeConstructor::TagCinematic => ("C", Color::rgb8(20, 184, 166)),
        NodeConstructor::Trade => ("T", Color::rgb8(245, 158, 11)),
        NodeConstructor::NestedDialog => ("N", Color::rgb8(139, 92, 246)),
        NodeConstructor::TagGreeting => ("G", Color::rgb8(16, 185, 129)),
        _ => ("?", Color::rgb8(156, 163, 175)),
    };

    label(move || text)
        .style(move |s| {
            s.font_size(10.0)
                .font_weight(Weight::BOLD)
                .padding_horiz(4.0)
                .padding_vert(2.0)
                .min_width(20.0)
                .background(bg_color)
                .color(Color::WHITE)
                .border_radius(3.0)
        })
}
