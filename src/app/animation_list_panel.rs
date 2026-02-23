// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use egui_ltreeview::{NodeBuilder, TreeView};
use egui_phosphor::regular::{ARROW_DOWN, ARROW_UP, MINUS, PENCIL_SIMPLE, PLUS, TRASH};

use super::ZephyrApp;
use super::settings_panel::{render_collapsing_section, two_column_row};
use crate::types::{Animation, PlaybackDirection};

enum Action {
    Select(usize),
    Rename(usize),
    Delete(usize),
}

pub(super) fn render(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    ui.add_space(4.0);

    let has_selection = app.selected_animation_idx.is_some();
    let can_move_up = app.selected_animation_idx.is_some_and(|i| i > 0);
    let can_move_down = app.selected_animation_idx.is_some_and(|i| i < app.animations.len().saturating_sub(1));

    egui::Frame::default()
        .inner_margin(egui::Margin { left: 4, right: 4, top: 0, bottom: 0 })
        .show(ui, |ui| {
            ui.columns(4, |columns| {
                columns[0].with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                    if ui.button(format!("{} Add", PLUS)).clicked() {
                        app.open_new_animation_modal();
                    }
                });

                columns[1].with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                    ui.add_enabled_ui(has_selection, |ui| {
                        if ui.button(format!("{} Remove", MINUS)).clicked() {
                            if let Some(idx) = app.selected_animation_idx {
                                delete_animation_at(app, idx, ui.ctx());
                            }
                        }
                    });
                });

                columns[2].with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                    ui.add_enabled_ui(can_move_up, |ui| {
                        if ui.button(ARROW_UP).clicked() {
                            if let Some(idx) = app.selected_animation_idx {
                                app.animations.swap(idx, idx - 1);
                                app.has_unsaved_changes = true;
                                let new_idx = idx - 1;
                                app.selected_animation_idx = Some(new_idx);
                                sync_tree_selection(ui.ctx(), new_idx);
                            }
                        }
                    });
                });

                columns[3].with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                    ui.add_enabled_ui(can_move_down, |ui| {
                        if ui.button(ARROW_DOWN).clicked() {
                            if let Some(idx) = app.selected_animation_idx {
                                app.animations.swap(idx, idx + 1);
                                app.has_unsaved_changes = true;
                                let new_idx = idx + 1;
                                app.selected_animation_idx = Some(new_idx);
                                sync_tree_selection(ui.ctx(), new_idx);
                            }
                        }
                    });
                });
            });
        });

    ui.add_space(4.0);

    render_animation_list(app, ui);

    if app.selected_animation_idx.is_some() {
        let margin = ui.style().spacing.window_margin;

        render_collapsing_section(ui, "Animation Settings", margin, |ui| {
            render_animation_settings(app, ui);
        });

        let has_frames = app
            .selected_animation_idx
            .and_then(|idx| app.animations.get(idx))
            .map(|a| !a.frames.is_empty())
            .unwrap_or(false);

        if has_frames {
            render_collapsing_section(ui, "Frame Settings", margin, |ui| {
                render_frame_settings(app, ui);
            });
        }
    }
}

pub(super) fn render_new_animation_modal(app: &mut ZephyrApp, ui_ctx: &egui::Context) {
    if !app.show_new_animation_modal {
        return;
    }

    egui::Window::new("New Animation")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ui_ctx, |ui| {
            ui.label("Animation name:");
            let response = ui.text_edit_singleline(&mut app.new_animation_name);

            let trimmed = app.new_animation_name.trim().to_string();
            let is_empty = trimmed.is_empty();
            let is_duplicate = !is_empty && animation_name_taken(&app.animations, &trimmed, None);
            let name_ok = !is_empty && !is_duplicate;

            if is_duplicate {
                ui.colored_label(ui.visuals().error_fg_color, "An animation with this name already exists.");
            }

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && name_ok {
                if let Some(new_idx) = app.finalize_new_animation() {
                    sync_tree_selection(ui_ctx, new_idx);
                }
            }

            if !response.has_focus() {
                response.request_focus();
            }

            ui.horizontal(|ui| {
                if ui.add_enabled(name_ok, egui::Button::new("Create")).clicked() {
                    if let Some(new_idx) = app.finalize_new_animation() {
                        sync_tree_selection(ui_ctx, new_idx);
                    }
                }
                if ui.button("Cancel").clicked() {
                    app.show_new_animation_modal = false;
                }
            });
        });
}

pub(super) fn render_animation_rename_modal(app: &mut ZephyrApp, ctx: &egui::Context) {
    let Some(renaming_idx) = app.renaming_animation_idx else {
        return;
    };

    egui::Window::new("Rename Animation")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label("Enter new name:");
            let response = ui.text_edit_singleline(&mut app.animation_rename_buffer);

            let trimmed = app.animation_rename_buffer.trim().to_string();
            let is_empty = trimmed.is_empty();

            // Exclude the animation being renamed so keeping the same name is accepted.
            let is_duplicate = !is_empty && animation_name_taken(&app.animations, &trimmed, Some(renaming_idx));
            let name_ok = !is_empty && !is_duplicate;

            if is_duplicate {
                ui.colored_label(ui.visuals().error_fg_color, "An animation with this name already exists.");
            }

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && name_ok {
                app.finalize_rename_animation();
            }

            if !response.has_focus() {
                response.request_focus();
            }

            ui.horizontal(|ui| {
                if ui.add_enabled(name_ok, egui::Button::new("Rename")).clicked() {
                    app.finalize_rename_animation();
                }
                if ui.button("Cancel").clicked() {
                    app.renaming_animation_idx = None;
                }
            });
        });
}

fn render_animation_list(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    let mut action: Option<Action> = None;
    let anim_count = app.animations.len();
    let tree_id = egui::Id::new("animation_tree");

    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        let (tree_response, tree_actions) = TreeView::new(tree_id).allow_multi_selection(false).show(ui, |builder| {
            for i in 0..anim_count {
                let name = match app.animations.get(i) {
                    Some(a) => a.name.clone(),
                    None => continue,
                };

                let node = NodeBuilder::leaf(i).label(name.as_str()).context_menu(|ui| {
                    ui.set_min_width(150.0);
                    ui.label(&name);
                    ui.separator();
                    if ui.button(format!("{} Rename", PENCIL_SIMPLE)).clicked() {
                        action = Some(Action::Rename(i));
                        ui.close();
                    }
                    if ui.button(format!("{} Delete", TRASH)).clicked() {
                        action = Some(Action::Delete(i));
                        ui.close();
                    }
                });
                builder.node(node);
            }
        });

        for tv_action in tree_actions {
            match tv_action {
                egui_ltreeview::Action::SetSelected(sel) => {
                    if let Some(&idx) = sel.first() {
                        action = Some(Action::Select(idx));
                    }
                }
                egui_ltreeview::Action::Activate(activate) => {
                    if let Some(&idx) = activate.selected.first() {
                        action = Some(Action::Rename(idx));
                    }
                }
                _ => {}
            }
        }

        // Only process Delete when the animation list is focused/hovered, an animation is
        // selected, and no modal is open.
        let list_is_focused = tree_response.hovered();
        let has_selection = app.selected_animation_idx.is_some();
        let no_modal_open = !app.show_new_animation_modal && app.renaming_animation_idx.is_none();

        if list_is_focused && has_selection && no_modal_open {
            if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
                if let Some(idx) = app.selected_animation_idx {
                    action = Some(Action::Delete(idx));
                }
            }
        }
    });

    match action {
        Some(Action::Select(idx)) => {
            if app.selected_animation_idx != Some(idx) {
                app.stop_playback();
                app.selected_animation_idx = Some(idx);
                app.anim_current_frame = 0;
            }
        }
        Some(Action::Rename(idx)) => {
            app.start_rename_animation(idx);
        }
        Some(Action::Delete(idx)) => {
            delete_animation_at(app, idx, ui.ctx());
        }
        None => {}
    }
}

fn render_animation_settings(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    let Some(anim_idx) = app.selected_animation_idx else {
        return;
    };
    let Some(anim) = app.animations.get_mut(anim_idx) else {
        return;
    };

    let old_direction = anim.direction;
    let old_loop = anim.loop_enabled;

    two_column_row(ui, "Direction:", |ui| {
        egui::ComboBox::from_id_salt("anim_direction")
            .width(ui.available_width())
            .selected_text(anim.direction.name())
            .show_ui(ui, |ui| {
                for dir in [PlaybackDirection::Forward, PlaybackDirection::Reverse, PlaybackDirection::PingPong] {
                    ui.selectable_value(&mut anim.direction, dir, dir.name()).on_hover_text(dir.description());
                }
            });
    });

    ui.add_space(4.0);

    two_column_row(ui, "Loop:", |ui| {
        ui.checkbox(&mut anim.loop_enabled, "")
            .on_hover_text("When enabled, playback repeats from the beginning after reaching the end");
    });

    if anim.direction != old_direction || anim.loop_enabled != old_loop {
        app.has_unsaved_changes = true;
    }
}

fn render_frame_settings(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    let Some(anim_idx) = app.selected_animation_idx else {
        return;
    };
    let frame_idx = app.anim_current_frame;

    let Some(frame) = app.animations.get_mut(anim_idx).and_then(|a| a.frames.get_mut(frame_idx)) else {
        return;
    };

    let mut delay_changed = false;
    two_column_row(ui, "Frame Delay:", |ui| {
        delay_changed = ui
            .add(egui::DragValue::new(&mut frame.delay_ms).speed(10).range(1..=60_000).suffix(" ms"))
            .on_hover_text("How long this frame is displayed during playback")
            .changed();
    });

    if delay_changed {
        app.has_unsaved_changes = true;
    }
}

fn delete_animation_at(app: &mut ZephyrApp, idx: usize, ctx: &egui::Context) {
    if idx >= app.animations.len() {
        return;
    }
    app.animations.remove(idx);
    app.has_unsaved_changes = true;
    app.stop_playback();
    let new_sel = if app.animations.is_empty() { None } else { Some(idx.saturating_sub(1)) };
    app.selected_animation_idx = new_sel;
    app.anim_current_frame = 0;
    if let Some(sel) = new_sel {
        sync_tree_selection(ctx, sel);
    } else {
        clear_tree_selection(ctx);
    }
}

fn animation_name_taken(animations: &[Animation], candidate: &str, exclude_idx: Option<usize>) -> bool {
    let lower = candidate.to_lowercase();
    animations.iter().enumerate().any(|(i, a)| exclude_idx != Some(i) && a.name.to_lowercase() == lower)
}

fn sync_tree_selection(ctx: &egui::Context, idx: usize) {
    let tree_id = egui::Id::new("animation_tree");
    ctx.data_mut(|data| {
        let state = data.get_persisted_mut_or_default::<egui_ltreeview::TreeViewState<usize>>(tree_id);
        state.set_selected(vec![idx]);
    });
}

fn clear_tree_selection(ctx: &egui::Context) {
    let tree_id = egui::Id::new("animation_tree");
    ctx.data_mut(|data| {
        let state = data.get_persisted_mut_or_default::<egui_ltreeview::TreeViewState<usize>>(tree_id);
        state.set_selected(Vec::<usize>::new());
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Animation;

    fn make_animations(names: &[&str]) -> Vec<Animation> {
        names.iter().map(|n| Animation::new(n.to_string())).collect()
    }

    #[test]
    fn unique_name_is_not_taken() {
        let anims = make_animations(&["Walk", "Run"]);
        assert!(!animation_name_taken(&anims, "Jump", None));
    }

    #[test]
    fn exact_match_is_taken() {
        let anims = make_animations(&["Walk", "Run"]);
        assert!(animation_name_taken(&anims, "Walk", None));
    }

    #[test]
    fn case_insensitive_match_is_taken() {
        let anims = make_animations(&["Walk", "Run"]);
        assert!(animation_name_taken(&anims, "WALK", None));
        assert!(animation_name_taken(&anims, "wAlK", None));
        assert!(animation_name_taken(&anims, "run", None));
    }

    #[test]
    fn excluded_index_is_not_counted() {
        let anims = make_animations(&["Walk", "Run"]);
        assert!(!animation_name_taken(&anims, "Walk", Some(0)));
        assert!(!animation_name_taken(&anims, "walk", Some(0)));
    }

    #[test]
    fn excluded_index_does_not_suppress_other_conflicts() {
        let anims = make_animations(&["Walk", "Run"]);
        assert!(animation_name_taken(&anims, "Run", Some(0)));
    }

    #[test]
    fn empty_list_is_never_taken() {
        assert!(!animation_name_taken(&[], "Anything", None));
    }
}
