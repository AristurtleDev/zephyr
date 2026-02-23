// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use egui_phosphor::regular::{CHECK_CIRCLE, LINE_SEGMENTS, TRASH, X_CIRCLE};

use super::ZephyrApp;
use super::settings_panel::{render_collapsing_section, two_column_row};
use crate::geometry;
use crate::types::{HitboxShape, HitboxType, PivotPoint, PivotPreset, SpriteProperties};

const PIVOT_PRESETS: &[PivotPreset] = &[
    PivotPreset::TopLeft,
    PivotPreset::TopCenter,
    PivotPreset::TopRight,
    PivotPreset::MiddleLeft,
    PivotPreset::Center,
    PivotPreset::MiddleRight,
    PivotPreset::BottomLeft,
    PivotPreset::BottomCenter,
    PivotPreset::BottomRight,
];

pub(super) fn render(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    let sprite_name = match app.selected_sprite_name() {
        Some(name) => name,
        None => {
            ui.add_space(12.0);
            ui.vertical_centered(|ui| {
                ui.label("Select a single sprite to edit its properties.");
            });
            return;
        }
    };

    let (sprite_w, sprite_h) = app
        .atlas
        .as_ref()
        .and_then(|a| a.placements.iter().find(|p| p.name == sprite_name))
        .map(|p| (p.width as f32, p.height as f32))
        .unwrap_or((64.0, 64.0));

    if !app.sprite_properties.contains_key(&sprite_name) {
        app.sprite_properties.insert(sprite_name.clone(), SpriteProperties::default());
    }

    let margin = ui.style().spacing.window_margin;

    ui.add_space(6.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        render_collapsing_section(ui, "Pivot Point", margin, |ui| {
            render_pivot_section(app, ui, &sprite_name);
        });
        render_collapsing_section(ui, "Hitbox", margin, |ui| {
            render_hitbox_section(app, ui, &sprite_name, sprite_w, sprite_h);
        });
    });
}

fn render_pivot_section(app: &mut ZephyrApp, ui: &mut egui::Ui, sprite_name: &str) {
    let Some(props) = app.sprite_properties.get_mut(sprite_name) else {
        return;
    };

    let old_enabled = props.pivot_enabled;

    two_column_row(ui, "Enable:", |ui| {
        ui.checkbox(&mut props.pivot_enabled, "");
    });

    let now_enabled = props.pivot_enabled;

    if !now_enabled {
        let enabled_changed = now_enabled != old_enabled;
        if enabled_changed {
            app.has_unsaved_changes = true;
        }
        return;
    }

    ui.add_space(4.0);

    let current_preset = props.pivot.preset;
    let mut new_preset = current_preset;

    two_column_row(ui, "Preset:", |ui| {
        egui::ComboBox::from_id_salt("pivot_preset")
            .width(ui.available_width())
            .selected_text(current_preset.name())
            .show_ui(ui, |ui| {
                for &preset in PIVOT_PRESETS {
                    ui.selectable_value(&mut new_preset, preset, preset.name());
                }
                ui.selectable_value(&mut new_preset, PivotPreset::Custom, PivotPreset::Custom.name());
            });
    });

    let preset_changed = new_preset != current_preset;
    let enabled_changed = now_enabled != old_enabled;
    if preset_changed {
        props.pivot.apply_preset(new_preset);
    }

    ui.add_space(4.0);

    let Some(props) = app.sprite_properties.get_mut(sprite_name) else {
        if enabled_changed || preset_changed {
            app.has_unsaved_changes = true;
        }
        return;
    };
    let PivotPoint { x, y, preset } = &mut props.pivot;
    let mut x_changed = false;
    let mut y_changed = false;

    two_column_row(ui, "X:", |ui| {
        x_changed = ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::DragValue::new(x).speed(0.01).range(0.0..=1.0).fixed_decimals(3),
            )
            .changed();
    });

    ui.add_space(4.0);

    two_column_row(ui, "Y:", |ui| {
        y_changed = ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::DragValue::new(y).speed(0.01).range(0.0..=1.0).fixed_decimals(3),
            )
            .changed();
    });

    if x_changed || y_changed {
        *preset = PivotPreset::Custom;
    }

    if enabled_changed || preset_changed || x_changed || y_changed {
        app.has_unsaved_changes = true;
    }
}

fn render_hitbox_section(app: &mut ZephyrApp, ui: &mut egui::Ui, sprite_name: &str, sprite_w: f32, sprite_h: f32) {
    let Some(props) = app.sprite_properties.get_mut(sprite_name) else {
        return;
    };

    let was_enabled = props.hitbox_enabled;
    two_column_row(ui, "Enable:", |ui| {
        ui.checkbox(&mut props.hitbox_enabled, "");
    });

    let now_enabled = props.hitbox_enabled;
    let enabled_changed = now_enabled != was_enabled;

    if !now_enabled {
        if enabled_changed {
            app.has_unsaved_changes = true;
        }
        return;
    }

    if !was_enabled {
        props.hitbox = HitboxShape::default_for_type(HitboxType::Rectangle, sprite_w, sprite_h);
    }

    ui.add_space(4.0);

    let current_type = props.hitbox.hitbox_type();
    let mut new_type = current_type;

    two_column_row(ui, "Shape:", |ui| {
        egui::ComboBox::from_id_salt("hitbox_type")
            .width(ui.available_width())
            .selected_text(current_type.name())
            .show_ui(ui, |ui| {
                for htype in [HitboxType::Rectangle, HitboxType::Circle, HitboxType::Polygon] {
                    ui.selectable_value(&mut new_type, htype, htype.name());
                }
            });
    });

    let type_changed = new_type != current_type;
    if type_changed {
        props.hitbox = HitboxShape::default_for_type(new_type, sprite_w, sprite_h);
        if new_type != HitboxType::Polygon {
            app.polygon_in_progress.clear();
            app.is_drawing_polygon = false;
        }
    }

    if enabled_changed || type_changed {
        app.has_unsaved_changes = true;
    }

    ui.add_space(4.0);

    let Some(props) = app.sprite_properties.get_mut(sprite_name) else {
        return;
    };

    // Capture bools before the match so all polygon action rows live in the same
    // two-column area as the Shape dropdown
    let is_polygon = matches!(&props.hitbox, HitboxShape::Polygon { .. });
    let is_polygon_with_points = matches!(&props.hitbox, HitboxShape::Polygon { points } if !points.is_empty());
    let is_drawing = app.is_drawing_polygon;
    let in_progress_count = app.polygon_in_progress.len();

    let mut clear_polygon = false;
    let mut finish_polygon = false;
    let mut cancel_draw = false;
    let mut start_draw = false;

    if is_polygon {
        const LABEL_W: f32 = 110.0;
        let interact_h = ui.spacing().interact_size.y;

        if is_drawing {
            ui.horizontal(|ui| {
                ui.add_enabled_ui(in_progress_count >= 3, |ui| {
                    if ui.add_sized([LABEL_W, interact_h], egui::Button::new(format!("{} Finish", CHECK_CIRCLE))).clicked() {
                        finish_polygon = true;
                    }
                });
                ui.add_space(8.0);
                if ui
                    .add_sized([ui.available_width(), interact_h], egui::Button::new(format!("{} Cancel", X_CIRCLE)))
                    .clicked()
                {
                    cancel_draw = true;
                }
            });
        } else if is_polygon_with_points {
            // Polygon exists: Clear fills the right column.
            two_column_row(ui, "", |ui| {
                if ui
                    .add_sized([ui.available_width(), interact_h], egui::Button::new(format!("{} Clear", TRASH)))
                    .clicked()
                {
                    clear_polygon = true;
                }
            });
        } else {
            // No polygon yet: Draw fills the right column.
            two_column_row(ui, "", |ui| {
                if ui
                    .add_sized([ui.available_width(), interact_h], egui::Button::new(format!("{} Draw", LINE_SEGMENTS)))
                    .clicked()
                {
                    start_draw = true;
                }
            });
        }
    }

    // Apply mutations that don't require `points` before entering the match.
    if cancel_draw {
        app.polygon_in_progress.clear();
        app.is_drawing_polygon = false;
    }
    if start_draw {
        app.polygon_in_progress.clear();
        app.is_drawing_polygon = true;
    }

    let inputs_changed = match &mut props.hitbox {
        HitboxShape::Rectangle { x, y, w, h } => render_rect_hitbox_inputs(ui, x, y, w, h, sprite_w, sprite_h),
        HitboxShape::Circle { cx, cy, radius } => render_circle_hitbox_inputs(ui, cx, cy, radius, sprite_w, sprite_h),
        HitboxShape::Polygon { points } => {
            let mut changed = false;
            // finish_polygon and clear_polygon need `points`; all other mutations
            // were already applied above before the match.
            if finish_polygon && in_progress_count >= 3 {
                let completed: Vec<[f32; 2]> = app.polygon_in_progress.drain(..).collect();
                app.is_drawing_polygon = false;
                *points = completed;
                changed = true;
            }
            if clear_polygon {
                points.clear();
                changed = true;
            }

            if app.is_drawing_polygon {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "Click on the sprite preview to add points.\n\
                         Click the first point to close the shape.",
                    )
                    .color(egui::Color32::LIGHT_BLUE),
                );
            }

            if !points.is_empty() && !app.is_drawing_polygon {
                ui.add_space(4.0);

                // Pre-compute column widths so headers and rows align.
                let spacing_x = ui.spacing().item_spacing.x;
                let interact_h = ui.spacing().interact_size.y;
                let available_w = ui.available_width();
                let idx_w = 24.0;
                let del_w = 22.0;
                let dv_w = ((available_w - idx_w - del_w - spacing_x * 3.0) / 2.0).max(30.0);

                ui.horizontal(|ui| {
                    ui.add_space(idx_w + spacing_x);
                    ui.allocate_ui_with_layout(egui::vec2(dv_w, 14.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                        ui.weak("X");
                    });
                    ui.allocate_ui_with_layout(egui::vec2(dv_w, 14.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                        ui.weak("Y");
                    });
                });
                ui.separator();

                let mut remove_idx: Option<usize> = None;
                for i in 0..points.len() {
                    ui.push_id(i, |ui| {
                        ui.horizontal(|ui| {
                            let mut temp = points[i];
                            ui.add_sized([idx_w, interact_h], egui::Label::new(egui::RichText::new(format!("{}", i)).weak()));
                            let x_resp = ui.add_sized([dv_w, interact_h], egui::DragValue::new(&mut temp[0]).speed(0.5).range(0.0..=sprite_w));
                            let y_resp = ui.add_sized([dv_w, interact_h], egui::DragValue::new(&mut temp[1]).speed(0.5).range(0.0..=sprite_h));
                            if x_resp.changed() || y_resp.changed() {
                                let mut candidate = points.clone();
                                candidate[i] = temp;
                                // Only commit if the edit doesn't create a self-intersecting polygon;
                                // this mirrors the reject-on-drag behavior in the sprite preview.
                                if !geometry::polygon_is_self_intersecting(&candidate) {
                                    points[i] = temp;
                                    changed = true;
                                }
                            }
                            if ui.add_sized([del_w, interact_h], egui::Button::new(X_CIRCLE)).clicked() {
                                remove_idx = Some(i);
                            }
                        });
                    });
                }
                if let Some(idx) = remove_idx {
                    points.remove(idx);
                    changed = true;
                }
            }

            changed
        }
    };

    if inputs_changed {
        app.has_unsaved_changes = true;
    }
}

fn render_rect_hitbox_inputs(ui: &mut egui::Ui, x: &mut f32, y: &mut f32, w: &mut f32, h: &mut f32, sprite_w: f32, sprite_h: f32) -> bool {
    let mut changed = false;
    two_column_row(ui, "X Offset:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::DragValue::new(x).speed(0.5).range(0.0..=sprite_w).suffix(" px"),
            )
            .changed()
        {
            changed = true;
        }
    });
    ui.add_space(4.0);
    two_column_row(ui, "Y Offset:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::DragValue::new(y).speed(0.5).range(0.0..=sprite_h).suffix(" px"),
            )
            .changed()
        {
            changed = true;
        }
    });
    ui.add_space(4.0);
    two_column_row(ui, "Width:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::DragValue::new(w).speed(0.5).range(1.0..=sprite_w).suffix(" px"),
            )
            .changed()
        {
            changed = true;
        }
    });
    ui.add_space(4.0);
    two_column_row(ui, "Height:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::DragValue::new(h).speed(0.5).range(1.0..=sprite_h).suffix(" px"),
            )
            .changed()
        {
            changed = true;
        }
    });

    let (cx, cy, cw, ch) = geometry::clamp_rect_to_bounds(*x, *y, *w, *h, sprite_w, sprite_h);
    *x = cx;
    *y = cy;
    *w = cw;
    *h = ch;

    changed
}

fn render_circle_hitbox_inputs(ui: &mut egui::Ui, cx: &mut f32, cy: &mut f32, radius: &mut f32, sprite_w: f32, sprite_h: f32) -> bool {
    let mut changed = false;
    let max_r = (sprite_w.min(sprite_h) / 2.0).max(1.0);
    two_column_row(ui, "Centre X:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::DragValue::new(cx).speed(0.5).range(0.0..=sprite_w).suffix(" px"),
            )
            .changed()
        {
            changed = true;
        }
    });
    ui.add_space(4.0);
    two_column_row(ui, "Centre Y:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::DragValue::new(cy).speed(0.5).range(0.0..=sprite_h).suffix(" px"),
            )
            .changed()
        {
            changed = true;
        }
    });
    ui.add_space(4.0);
    two_column_row(ui, "Radius:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::DragValue::new(radius).speed(0.5).range(1.0..=max_r).suffix(" px"),
            )
            .changed()
        {
            changed = true;
        }
    });

    let (ncx, ncy, nr) = geometry::clamp_circle_to_bounds(*cx, *cy, *radius, sprite_w, sprite_h);
    *cx = ncx;
    *cy = ncy;
    *radius = nr;

    changed
}
