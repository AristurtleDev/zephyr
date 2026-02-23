// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use egui::{Color32, Pos2, Rect, Stroke, Ui, vec2};
use egui_phosphor::regular::{ARROW_FAT_LEFT, ARROW_FAT_RIGHT, PAUSE, PLAY, SKIP_BACK, SKIP_FORWARD, TRASH};

use super::ZephyrApp;
use super::state::BackgroundStyle;
use crate::types::AnimationFrame;

const THUMB_SIZE: f32 = 64.0;

pub(super) fn render_content(app: &mut ZephyrApp, ui: &mut Ui, ctx: &egui::Context) {
    app.tick_anim_playback(ctx);

    if app.selected_animation_idx.is_none() || app.animations.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.label("Select or create an animation in the panel on the right.");
        });
        return;
    }

    let Some(anim_idx) = app.selected_animation_idx else {
        return;
    };

    // If a sprite drag was released while the pointer is over this panel,
    // append it as a new frame.  ui.clip_rect() is the bounding rect of the
    // central panel, so this only fires when the pointer is inside the
    // animation tab
    let panel_rect = ui.clip_rect();
    let released_here = ctx.input(|i| i.pointer.any_released() && i.pointer.hover_pos().map(|p| panel_rect.contains(p)).unwrap_or(false));
    if released_here {
        if let Some(names) = egui::DragAndDrop::payload::<super::SpriteDragPayload>(ctx).map(|p| p.0.clone()) {
            if let Some(anim) = app.animations.get_mut(anim_idx) {
                for name in &names {
                    anim.frames.push(AnimationFrame::new(name));
                }
            }
            egui::DragAndDrop::clear_payload(ctx);
            app.stop_playback();
            app.has_unsaved_changes = true;
            let fc = app.animations.get(anim_idx).map_or(0, |a| a.frames.len());
            app.anim_current_frame = fc.saturating_sub(1);
        }
    }

    let available_height = ui.available_height();
    let timeline_height = (THUMB_SIZE + 56.0).max(1.0);
    let controls_height = ui.spacing().interact_size.y + 8.0;
    let preview_height = (available_height - timeline_height - controls_height - 16.0).max(60.0);

    render_playback_preview(app, ui, anim_idx, preview_height);
    ui.separator();
    render_playback_controls(app, ui, anim_idx);
    render_frame_timeline(app, ui, ctx, anim_idx);
}

fn render_playback_preview(app: &mut ZephyrApp, ui: &mut Ui, anim_idx: usize, height: f32) {
    let available_w = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(available_w, height), egui::Sense::click_and_drag());

    ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(32, 32, 32));

    let frame_count = app.animations.get(anim_idx).map_or(0, |a| a.frames.len());

    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            let old_zoom = app.anim_preview_zoom;
            let factor = if scroll > 0.0 { 1.1 } else { 0.9 };
            app.anim_preview_zoom = (app.anim_preview_zoom * factor).clamp(0.1, 20.0);

            if let Some(mouse) = response.hover_pos() {
                let center = rect.center() + app.anim_preview_pan;
                let delta = mouse - center;
                let ratio = app.anim_preview_zoom / old_zoom;
                app.anim_preview_pan += delta * (1.0 - ratio);
            }
        }
    }

    if response.dragged_by(egui::PointerButton::Middle) {
        app.anim_preview_pan += response.drag_delta();
    }

    if frame_count == 0 {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No frames",
            egui::FontId::proportional(14.0),
            Color32::DARK_GRAY,
        );
        return;
    }

    let current = app.anim_current_frame.min(frame_count - 1);
    let sprite_name = app
        .animations
        .get(anim_idx)
        .and_then(|a| a.frames.get(current))
        .map(|f| f.sprite_name.clone())
        .unwrap_or_default();

    if let (Some(atlas_tex), Some(atlas)) = (&app.atlas_texture, &app.atlas) {
        // Use source (pre-trim) dimensions so the checkerboard stays stable
        // across frames even when trim mode is active.
        let (max_w, max_h) = app
            .animations
            .get(anim_idx)
            .map(|anim| {
                anim.frames
                    .iter()
                    .filter_map(|frame| {
                        atlas
                            .placements
                            .iter()
                            .find(|p| p.name == frame.sprite_name)
                            .map(|p| (p.source_width as f32, p.source_height as f32))
                    })
                    .fold((0.0_f32, 0.0_f32), |(max_w, max_h), (w, h)| (max_w.max(w), max_h.max(h)))
            })
            .unwrap_or((64.0, 64.0));

        // Calculate a single scale factor based on fitting the largest sprite.
        // All sprites will use this same scale to maintain relative sizing.
        let max_aspect = max_w / max_h.max(1.0);
        let base_scale = if max_aspect > rect.width() / rect.height() {
            // Constrained by width
            (rect.width() * 0.9) / max_w
        } else {
            // Constrained by height
            (rect.height() * 0.9) / max_h
        };

        let scale = base_scale * app.anim_preview_zoom;

        let checker_dw = max_w * scale;
        let checker_dh = max_h * scale;
        let center = rect.center() + app.anim_preview_pan;
        let checker_rect = Rect::from_center_size(center, vec2(checker_dw, checker_dh));

        match app.prefs.background_style {
            BackgroundStyle::Checkerboard => super::draw_checker(ui, &checker_rect, &rect, 8.0),
            BackgroundStyle::SolidColor => {
                ui.painter().with_clip_rect(rect).rect_filled(checker_rect, 0.0, app.prefs.solid_bg_color);
            }
            BackgroundStyle::Off => {}
        }

        if let Some(placement) = atlas.placements.iter().find(|p| p.name == sprite_name) {
            let aw = atlas.width as f32;
            let ah = atlas.height as f32;
            let uv = Rect::from_min_max(
                Pos2::new(placement.x as f32 / aw, placement.y as f32 / ah),
                Pos2::new((placement.x + placement.width) as f32 / aw, (placement.y + placement.height) as f32 / ah),
            );

            let dw = placement.width as f32 * scale;
            let dh = placement.height as f32 * scale;

            // When a sprite is trimmed, position it at its original offset within
            // the source frame rather than centering it, so all frames of an
            // animation remain visually aligned.
            let sprite_min = checker_rect.min + vec2(placement.trim_offset_x as f32 * scale, placement.trim_offset_y as f32 * scale);
            let sprite_rect = Rect::from_min_size(sprite_min, vec2(dw, dh));

            ui.painter().with_clip_rect(rect).image(atlas_tex.id(), sprite_rect, uv, Color32::WHITE);
        }

        if app.prefs.draw_border {
            ui.painter()
                .with_clip_rect(rect)
                .rect_stroke(checker_rect, 0.0, (1.0, Color32::WHITE), egui::StrokeKind::Outside);
        }
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Pack atlas first",
            egui::FontId::proportional(14.0),
            Color32::DARK_GRAY,
        );
    }

    let label = format!("Frame {} / {}  ({})", current + 1, frame_count, sprite_name,);
    ui.painter().text(
        rect.left_bottom() + vec2(6.0, -6.0),
        egui::Align2::LEFT_BOTTOM,
        &label,
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
}

fn render_playback_controls(app: &mut ZephyrApp, ui: &mut Ui, anim_idx: usize) {
    let frame_count = app.animations.get(anim_idx).map_or(0, |a| a.frames.len());

    let button_width = 50.0;
    let button_height = ui.spacing().interact_size.y;
    let button_size = vec2(button_width, button_height);

    ui.horizontal(|ui| {
        if ui.add_sized(button_size, egui::Button::new(SKIP_BACK)).on_hover_text("First frame").clicked() {
            app.stop_playback();
            app.anim_current_frame = 0;
        }

        if ui
            .add_sized(button_size, egui::Button::new(ARROW_FAT_LEFT))
            .on_hover_text("Previous frame")
            .clicked()
        {
            app.stop_playback();
            if frame_count > 0 {
                app.anim_current_frame = (app.anim_current_frame + frame_count - 1) % frame_count;
            }
        }

        if app.anim_playing {
            if ui.add_sized(button_size, egui::Button::new(PAUSE)).on_hover_text("Pause").clicked() {
                app.anim_playing = false;
                app.anim_last_time = None;
            }
        } else if ui.add_sized(button_size, egui::Button::new(PLAY)).on_hover_text("Play").clicked() && frame_count > 0 {
            app.anim_playing = true;
            app.anim_frame_elapsed_ms = 0.0;
            app.anim_last_time = None;
            app.anim_ping_pong_forward = true;
        }

        if ui.add_sized(button_size, egui::Button::new(ARROW_FAT_RIGHT)).on_hover_text("Next frame").clicked() {
            app.stop_playback();
            if frame_count > 0 {
                app.anim_current_frame = (app.anim_current_frame + 1) % frame_count;
            }
        }

        if ui.add_sized(button_size, egui::Button::new(SKIP_FORWARD)).on_hover_text("Last frame").clicked() {
            app.stop_playback();
            if frame_count > 0 {
                app.anim_current_frame = frame_count - 1;
            }
        }
    });
}

fn render_frame_timeline(app: &mut ZephyrApp, ui: &mut Ui, ctx: &egui::Context, anim_idx: usize) {
    ui.separator();

    let frame_count = app.animations.get(anim_idx).map_or(0, |a| a.frames.len());

    let panel_rect = ui.clip_rect();
    let is_sprite_drag =
        egui::DragAndDrop::payload::<super::SpriteDragPayload>(ctx).is_some() && ctx.input(|i| i.pointer.hover_pos()).map(|p| panel_rect.contains(p)).unwrap_or(false);

    // When a sprite drag is active, draw a blue highlight border around the
    // whole timeline area so the user can see where to release.
    if is_sprite_drag {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 4.0, Color32::from_rgba_unmultiplied(80, 150, 255, 20));
        ui.painter()
            .rect_stroke(rect, 4.0, Stroke::new(1.5, Color32::from_rgb(80, 150, 255)), egui::StrokeKind::Inside);
    }

    let mut delete_frame: Option<usize> = None;
    let mut move_frame: Option<(usize, i32)> = None;
    let mut reorder_frame: Option<(usize, usize)> = None;

    if frame_count == 0 {
        let msg = if is_sprite_drag {
            "Release to add frame"
        } else {
            "No frames yet. Drag a sprite from the panel on the left."
        };
        ui.label(egui::RichText::new(msg).italics().color(if is_sprite_drag {
            Color32::from_rgb(100, 180, 255)
        } else {
            Color32::DARK_GRAY
        }));
        return;
    }

    let mut pending_reorder: Option<(usize, usize)> = None;

    let timeline_rect = ui.available_rect_before_wrap();

    egui::ScrollArea::horizontal().show(ui, |ui| {
        // Auto-scroll when dragging a frame near the left/right viewport edge.
        if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
            if egui::DragAndDrop::payload::<super::FrameDragPayload>(ctx).is_some() {
                let scroll_margin = 50.0;
                let scroll_speed = 10.0;
                let viewport = ui.clip_rect();
                if pos.x < viewport.min.x + scroll_margin {
                    ui.scroll_with_delta(vec2(scroll_speed, 0.0));
                } else if pos.x > viewport.max.x - scroll_margin {
                    ui.scroll_with_delta(vec2(-scroll_speed, 0.0));
                }
            }
        }

        ui.horizontal(|ui| {
            let frame_being_dragged = egui::DragAndDrop::payload::<super::FrameDragPayload>(ctx).map(|p| p.0);

            let mouse_pos = ctx.input(|i| i.pointer.hover_pos());
            let mut insertion_idx: Option<usize> = None;

            let mut dragged_frame_info: Option<(Rect, usize, bool)> = None;

            ui.add_space(8.0);

            for i in 0..frame_count {
                let is_dragged = frame_being_dragged == Some(i);

                ui.push_id(i, |ui| {
                    let is_current = i == app.anim_current_frame;

                    let (cell_rect, cell_response) = ui.allocate_exact_size(vec2(THUMB_SIZE + 4.0, THUMB_SIZE + 28.0), egui::Sense::click_and_drag());

                    if cell_response.drag_started() {
                        egui::DragAndDrop::set_payload(ctx, super::FrameDragPayload(i));
                        app.stop_playback();
                    }

                    if cell_response.clicked() {
                        app.stop_playback();
                        app.anim_current_frame = i;
                    }

                    cell_response.context_menu(|ui| {
                        ui.set_min_width(150.0);
                        ui.label(format!("Frame {}", i + 1));
                        ui.separator();

                        if ui.button(format!("{} Move Left", ARROW_FAT_LEFT)).clicked() {
                            move_frame = Some((i, -1));
                            ui.close();
                        }

                        if ui.button(format!("{} Move Right", ARROW_FAT_RIGHT)).clicked() {
                            move_frame = Some((i, 1));
                            ui.close();
                        }

                        ui.separator();

                        if ui.button(format!("{} Delete", TRASH)).clicked() {
                            delete_frame = Some(i);
                            ui.close();
                        }
                    });

                    let mut shift_x = 0.0;
                    if let Some(dragged_idx) = frame_being_dragged {
                        if let Some(pos) = mouse_pos {
                            if !is_dragged {
                                let frame_center = cell_rect.center().x;

                                if dragged_idx < i {
                                    if pos.x > frame_center {
                                        shift_x = -(THUMB_SIZE + 4.0);
                                        insertion_idx = Some(i + 1);
                                    }
                                } else {
                                    if pos.x < frame_center {
                                        shift_x = THUMB_SIZE + 4.0;
                                        if insertion_idx.is_none() || insertion_idx.unwrap() > i {
                                            insertion_idx = Some(i);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if frame_being_dragged.is_some() && insertion_idx.is_none() && i == frame_count - 1 {
                        insertion_idx = frame_being_dragged;
                    }

                    if is_dragged {
                        dragged_frame_info = Some((cell_rect, i, is_current));
                        return;
                    }

                    let render_rect = cell_rect.translate(vec2(shift_x, 0.0));

                    let bg = if is_current {
                        Color32::from_rgba_unmultiplied(80, 140, 220, 60)
                    } else if cell_response.hovered() {
                        Color32::from_rgba_unmultiplied(255, 255, 255, 15)
                    } else {
                        Color32::TRANSPARENT
                    };
                    ui.painter().rect_filled(render_rect, 4.0, bg);

                    if is_current {
                        ui.painter()
                            .rect_stroke(render_rect, 4.0, Stroke::new(1.5, Color32::from_rgb(80, 140, 220)), egui::StrokeKind::Outside);
                    }

                    let thumb_rect = Rect::from_min_size(render_rect.min + vec2(2.0, 2.0), vec2(THUMB_SIZE, THUMB_SIZE));

                    draw_frame_thumbnail(app, ui, anim_idx, i, &thumb_rect);

                    let delay_rect = Rect::from_min_size(render_rect.min + vec2(0.0, THUMB_SIZE + 4.0), vec2(THUMB_SIZE + 4.0, 20.0));
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(delay_rect), |ui| {
                        ui.horizontal(|ui| {
                            if let Some(frame) = app.animations.get_mut(anim_idx).and_then(|a| a.frames.get_mut(i)) {
                                if ui.add(egui::DragValue::new(&mut frame.delay_ms).speed(10).range(1..=60_000).suffix("ms")).changed() {
                                    app.has_unsaved_changes = true;
                                }
                            }
                        });
                    });
                });
            }

            if let Some((_original_rect, frame_idx, is_current)) = dragged_frame_info {
                if let Some(pos) = mouse_pos {
                    let drag_visual_size = vec2(THUMB_SIZE + 4.0, THUMB_SIZE + 4.0);

                    let dragged_rect = Rect::from_center_size(pos, drag_visual_size);
                    let bg = if is_current {
                        Color32::from_rgba_unmultiplied(80, 140, 220, 100)
                    } else {
                        Color32::from_rgba_unmultiplied(60, 60, 60, 150)
                    };

                    ui.painter().rect_filled(dragged_rect, 4.0, bg);
                    ui.painter()
                        .rect_stroke(dragged_rect, 4.0, Stroke::new(2.0, Color32::from_rgb(80, 150, 255)), egui::StrokeKind::Outside);

                    let thumb_rect = Rect::from_min_size(dragged_rect.min + vec2(2.0, 2.0), vec2(THUMB_SIZE, THUMB_SIZE));

                    draw_frame_thumbnail(app, ui, anim_idx, frame_idx, &thumb_rect);
                    ui.painter().text(
                        thumb_rect.left_top() + vec2(2.0, 2.0),
                        egui::Align2::LEFT_TOP,
                        (frame_idx + 1).to_string(),
                        egui::FontId::proportional(10.0),
                        Color32::WHITE,
                    );
                }
            }

            if let Some(dragged_idx) = frame_being_dragged {
                if ctx.input(|i| i.pointer.any_released()) {
                    if let Some(insert_at) = insertion_idx {
                        let would_move = insert_at != dragged_idx && insert_at != dragged_idx + 1;
                        if would_move {
                            pending_reorder = Some((dragged_idx, insert_at));
                        }
                    }
                    egui::DragAndDrop::clear_payload(ctx);
                }
            }
        });
    });

    handle_timeline_keyboard(app, ui, frame_count, timeline_rect, &mut delete_frame, &mut move_frame);

    if let Some((from_idx, to_idx)) = pending_reorder {
        reorder_frame = Some((from_idx, to_idx));
    }

    // Deferred mutations: applied after all borrows on app.animations are released.
    apply_timeline_mutations(app, anim_idx, delete_frame, move_frame, reorder_frame);
}

fn handle_timeline_keyboard(app: &mut ZephyrApp, ui: &mut Ui, frame_count: usize, timeline_rect: Rect, delete_frame: &mut Option<usize>, move_frame: &mut Option<(usize, i32)>) {
    let timeline_is_focused = ui.input(|i| i.pointer.hover_pos().map(|pos| timeline_rect.contains(pos)).unwrap_or(false));
    let has_current_frame = app.anim_current_frame < frame_count;

    if !timeline_is_focused || !has_current_frame {
        return;
    }

    let current = app.anim_current_frame;

    if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
        *delete_frame = Some(current);
    }

    if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft) && i.modifiers.alt) {
        *move_frame = Some((current, -1));
    }

    if ui.input(|i| i.key_pressed(egui::Key::ArrowRight) && i.modifiers.alt) {
        *move_frame = Some((current, 1));
    }

    // ArrowLeft/Right without Alt navigate frame selection; stop playback so
    // the selected frame is actually visible in the preview.
    if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft) && !i.modifiers.alt) {
        app.stop_playback();
        if current > 0 {
            app.anim_current_frame = current - 1;
        }
    }

    if ui.input(|i| i.key_pressed(egui::Key::ArrowRight) && !i.modifiers.alt) {
        app.stop_playback();
        if current < frame_count - 1 {
            app.anim_current_frame = current + 1;
        }
    }
}

fn apply_timeline_mutations(app: &mut ZephyrApp, anim_idx: usize, delete_frame: Option<usize>, move_frame: Option<(usize, i32)>, reorder_frame: Option<(usize, usize)>) {
    if let Some(idx) = delete_frame {
        if let Some(anim) = app.animations.get_mut(anim_idx) {
            anim.frames.remove(idx);
            app.has_unsaved_changes = true;
        }
        app.stop_playback();
        let fc = app.animations.get(anim_idx).map_or(0, |a| a.frames.len());
        if fc == 0 {
            app.anim_current_frame = 0;
        } else {
            app.anim_current_frame = app.anim_current_frame.min(fc - 1);
        }
    }

    if let Some((idx, dir)) = move_frame {
        let new_idx = idx as i32 + dir;
        if new_idx >= 0 {
            let new_idx = new_idx as usize;
            if let Some(frames) = app.animations.get_mut(anim_idx).map(|a| &mut a.frames)
                && new_idx < frames.len()
            {
                frames.swap(idx, new_idx);
                app.has_unsaved_changes = true;
                if app.anim_current_frame == idx {
                    app.anim_current_frame = new_idx;
                } else if app.anim_current_frame == new_idx {
                    app.anim_current_frame = idx;
                }
            }
        }
    }

    if let Some((from_idx, to_idx)) = reorder_frame {
        if let Some(anim) = app.animations.get_mut(anim_idx) {
            if from_idx < anim.frames.len() && to_idx <= anim.frames.len() && from_idx != to_idx {
                let frame = anim.frames.remove(from_idx);
                // Adjust insertion index if we removed an element before it
                let insert_idx = if from_idx < to_idx { to_idx - 1 } else { to_idx };
                anim.frames.insert(insert_idx, frame);
                app.has_unsaved_changes = true;

                // Update current frame tracking if it was affected
                if app.anim_current_frame == from_idx {
                    app.anim_current_frame = insert_idx;
                } else if from_idx < app.anim_current_frame && insert_idx >= app.anim_current_frame {
                    app.anim_current_frame = app.anim_current_frame.saturating_sub(1);
                } else if from_idx > app.anim_current_frame && insert_idx <= app.anim_current_frame {
                    app.anim_current_frame += 1;
                }
            }
        }
        app.stop_playback();
    }
}

fn draw_frame_thumbnail(app: &ZephyrApp, ui: &mut Ui, anim_idx: usize, frame_idx: usize, thumb_rect: &Rect) {
    super::draw_checker(ui, thumb_rect, thumb_rect, 8.0);

    let sprite_name: &str = match app.animations.get(anim_idx).and_then(|a| a.frames.get(frame_idx)) {
        Some(frame) => &frame.sprite_name,
        None => return,
    };

    if let (Some(atlas_tex), Some(atlas)) = (&app.atlas_texture, &app.atlas) {
        if let Some(placement) = atlas.placements.iter().find(|p| p.name == *sprite_name) {
            let aw = atlas.width as f32;
            let ah = atlas.height as f32;
            let uv = Rect::from_min_max(
                Pos2::new(placement.x as f32 / aw, placement.y as f32 / ah),
                Pos2::new((placement.x + placement.width) as f32 / aw, (placement.y + placement.height) as f32 / ah),
            );

            let aspect = placement.width as f32 / placement.height.max(1) as f32;
            let (dw, dh) = if aspect > 1.0 {
                let w = thumb_rect.width();
                (w, w / aspect)
            } else {
                let h = thumb_rect.height();
                (h * aspect, h)
            };
            let sprite_rect = Rect::from_center_size(thumb_rect.center(), vec2(dw, dh));
            ui.painter().image(atlas_tex.id(), sprite_rect, uv, Color32::WHITE);
        }
    } else {
        ui.painter().text(
            thumb_rect.center(),
            egui::Align2::CENTER_CENTER,
            sprite_name,
            egui::FontId::proportional(9.0),
            Color32::LIGHT_GRAY,
        );
    }

    ui.painter().text(
        thumb_rect.left_top() + vec2(2.0, 2.0),
        egui::Align2::LEFT_TOP,
        (frame_idx + 1).to_string(),
        egui::FontId::proportional(10.0),
        Color32::WHITE,
    );
}
