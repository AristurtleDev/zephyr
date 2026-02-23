// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use egui::{Color32, Pos2, Rect, Stroke, Ui, vec2};

use super::ZephyrApp;
use super::state::{BackgroundStyle, HitboxDragState, HitboxDragTarget};
use crate::geometry::{self, CircleHandle, RectHandle};
use crate::types::{HitboxShape, PivotPreset, SpriteProperties};

pub(super) fn render_content(app: &mut ZephyrApp, ui: &mut Ui, ctx: &egui::Context) {
    let sprite_name = match app.selected_sprite_name() {
        Some(name) => name,
        None => {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
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

    render_sprite_preview(app, ui, &sprite_name, sprite_w, sprite_h, ctx);

    if app.show_invalid_polygon_placement_modal {
        egui::Window::new("Invalid Placement")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label("Cannot place vertex here.");
                ui.label("Placing this vertex would create a self-intersecting polygon.");
                ui.add_space(8.0);
                if ui.button("OK").clicked() {
                    app.show_invalid_polygon_placement_modal = false;
                }
            });
    }
}

fn render_sprite_preview(app: &mut ZephyrApp, ui: &mut Ui, sprite_name: &str, sprite_w: f32, sprite_h: f32, ctx: &egui::Context) {
    let available = ui.available_size();
    let (full_rect, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());

    ui.painter().rect_filled(full_rect, 0.0, Color32::from_rgb(48, 48, 48));

    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            let old_zoom = app.sprite_editor_zoom;
            let factor = if scroll > 0.0 { 1.1 } else { 0.9 };
            app.sprite_editor_zoom = (app.sprite_editor_zoom * factor).clamp(0.1, 20.0);

            if let Some(mouse) = response.hover_pos() {
                let center = full_rect.center() + app.sprite_editor_pan;
                let delta = mouse - center;
                let ratio = app.sprite_editor_zoom / old_zoom;
                app.sprite_editor_pan += delta * (1.0 - ratio);
            }
        }
    }

    if response.dragged_by(egui::PointerButton::Middle) {
        app.sprite_editor_pan += response.drag_delta();
    }

    let base_scale = ((available.x * 0.8) / sprite_w).min((available.y * 0.8) / sprite_h).max(0.5);
    let scale = base_scale * app.sprite_editor_zoom;

    let display_w = sprite_w * scale;
    let display_h = sprite_h * scale;
    let center = full_rect.center() + app.sprite_editor_pan;
    let sprite_rect = Rect::from_center_size(center, vec2(display_w, display_h));

    match app.prefs.background_style {
        BackgroundStyle::Checkerboard => super::draw_checker(ui, &sprite_rect, &full_rect, 8.0),
        BackgroundStyle::SolidColor => {
            ui.painter().with_clip_rect(full_rect).rect_filled(sprite_rect, 0.0, app.prefs.solid_bg_color);
        }
        BackgroundStyle::Off => {}
    }

    if let (Some(atlas_tex), Some(atlas)) = (&app.atlas_texture, &app.atlas)
        && let Some(placement) = atlas.placements.iter().find(|p| p.name == sprite_name)
    {
        let aw = atlas.width as f32;
        let ah = atlas.height as f32;
        let uv = Rect::from_min_max(
            Pos2::new(placement.x as f32 / aw, placement.y as f32 / ah),
            Pos2::new((placement.x + placement.width) as f32 / aw, (placement.y + placement.height) as f32 / ah),
        );
        ui.painter().with_clip_rect(full_rect).image(atlas_tex.id(), sprite_rect, uv, Color32::WHITE);
    }

    if app.prefs.draw_border {
        ui.painter()
            .with_clip_rect(full_rect)
            .rect_stroke(sprite_rect, 0.0, (1.0, Color32::WHITE), egui::StrokeKind::Outside);
    }

    let is_pivot_hovered = !app.is_drawing_polygon
        && app.hitbox_drag.is_none()
        && response.hover_pos().is_some_and(|pos| {
            app.sprite_properties
                .get(sprite_name)
                .is_some_and(|p| p.pivot_enabled && { (pos - Pos2::new(sprite_rect.min.x + p.pivot.x * display_w, sprite_rect.min.y + p.pivot.y * display_h)).length_sq() <= 64.0 })
        });

    // Computed before the props borrow; only active when the hitbox is interactable.
    let hover_target: Option<HitboxDragTarget> = if !app.is_drawing_polygon && app.hitbox_drag.is_none() && !app.pivot_drag && !is_pivot_hovered {
        response.hover_pos().and_then(|hover_pos| {
            let lx = (hover_pos.x - sprite_rect.min.x) / scale;
            let ly = (hover_pos.y - sprite_rect.min.y) / scale;
            app.sprite_properties
                .get(sprite_name)
                .filter(|p| p.hitbox_enabled)
                .and_then(|p| find_drag_target(&p.hitbox, lx, ly, scale))
        })
    } else {
        None
    };

    if let Some(props) = app.sprite_properties.get(sprite_name) {
        if props.hitbox_enabled {
            draw_hitbox_overlay(ui, &sprite_rect, &full_rect, &props.hitbox, scale, &app.hitbox_drag, hover_target);
        }

        if props.pivot_enabled {
            let px = sprite_rect.min.x + props.pivot.x * display_w;
            let py = sprite_rect.min.y + props.pivot.y * display_h;
            let p = Pos2::new(px, py);
            let pivot_color = if is_pivot_hovered || app.pivot_drag {
                Color32::WHITE
            } else {
                Color32::from_rgb(255, 220, 0)
            };
            let stroke = Stroke::new(1.5, pivot_color);
            let painter = ui.painter().with_clip_rect(full_rect);
            painter.line_segment([Pos2::new(p.x - 8.0, p.y), Pos2::new(p.x + 8.0, p.y)], stroke);
            painter.line_segment([Pos2::new(p.x, p.y - 8.0), Pos2::new(p.x, p.y + 8.0)], stroke);
            painter.circle(p, 3.5, pivot_color, Stroke::NONE);
        }
    }

    handle_sprite_editor_interactions(app, ui, &response, sprite_name, sprite_rect, scale, sprite_w, sprite_h, full_rect, ctx);
}

fn handle_sprite_editor_interactions(
    app: &mut ZephyrApp,
    ui: &mut Ui,
    response: &egui::Response,
    sprite_name: &str,
    sprite_rect: Rect,
    scale: f32,
    sprite_w: f32,
    sprite_h: f32,
    full_rect: Rect,
    ctx: &egui::Context,
) {
    if !app.is_drawing_polygon {
        handle_pivot_drag(app, response, sprite_name, sprite_rect, scale, sprite_w, sprite_h, ctx);

        if !app.pivot_drag {
            handle_hitbox_drag(app, response, sprite_name, sprite_rect, scale, sprite_w, sprite_h, ctx);
        }

        // Double-click near a polygon edge inserts a new vertex at the closest point.
        if !app.pivot_drag
            && response.double_clicked()
            && let Some(pos) = response.interact_pointer_pos()
            && sprite_rect.contains(pos)
        {
            let (lx, ly, t) = ((pos.x - sprite_rect.min.x) / scale, (pos.y - sprite_rect.min.y) / scale, 8.0 / scale);
            let vertex_inserted = if let Some(pr) = app.sprite_properties.get_mut(sprite_name)
                && pr.hitbox_enabled
                && let HitboxShape::Polygon { points } = &mut pr.hitbox
                && geometry::polygon_vertex_hit_test(points, lx, ly, t).is_none()
                && let Some((i, c)) = geometry::polygon_closest_edge_point(points, lx, ly, t)
            {
                let new_pt = [c[0].round().clamp(0.0, sprite_w), c[1].round().clamp(0.0, sprite_h)];
                points.insert(i + 1, new_pt);
                true
            } else {
                false
            };
            if vertex_inserted {
                app.has_unsaved_changes = true;
                ctx.request_repaint();
            }
        }

        // Right-click near a polygon vertex opens a "Delete Node" context menu.
        if !app.pivot_drag && response.secondary_clicked() {
            let v = response.interact_pointer_pos().and_then(|pos| {
                let [lx, ly] = [(pos.x - sprite_rect.min.x) / scale, (pos.y - sprite_rect.min.y) / scale];
                app.sprite_properties.get(sprite_name).filter(|p| p.hitbox_enabled).and_then(|p| {
                    if let HitboxShape::Polygon { points } = &p.hitbox {
                        geometry::polygon_vertex_hit_test(points, lx, ly, 8.0 / scale)
                    } else {
                        None
                    }
                })
            });
            app.polygon_ctx_menu_vertex = v;
        }
        if app.polygon_ctx_menu_vertex.is_some() {
            let vi = app.polygon_ctx_menu_vertex;
            let pc = app
                .sprite_properties
                .get(sprite_name)
                .and_then(|p| {
                    if let HitboxShape::Polygon { points } = &p.hitbox {
                        Some(points.len())
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            response.context_menu(|ui| {
                if let Some(idx) = vi
                    && ui.add_enabled(pc > 3, egui::Button::new("Delete Node")).clicked()
                {
                    let vertex_removed = if let Some(pr) = app.sprite_properties.get_mut(sprite_name)
                        && let HitboxShape::Polygon { points } = &mut pr.hitbox
                        && idx < points.len()
                    {
                        points.remove(idx);
                        true
                    } else {
                        false
                    };
                    if vertex_removed {
                        app.has_unsaved_changes = true;
                    }
                    app.polygon_ctx_menu_vertex = None;
                    ui.close();
                }
            });
        }
    }

    if app.is_drawing_polygon {
        let pts: Vec<Pos2> = app
            .polygon_in_progress
            .iter()
            .map(|&[x, y]| Pos2::new(sprite_rect.min.x + x * scale, sprite_rect.min.y + y * scale))
            .collect();

        let stroke = Stroke::new(1.5, Color32::from_rgb(100, 200, 255));
        let painter = ui.painter().with_clip_rect(full_rect);

        for window in pts.windows(2) {
            if let [a, b] = window {
                painter.line_segment([*a, *b], stroke);
            }
        }
        if pts.len() >= 2 {
            if let (Some(&last_pt), Some(&first_pt)) = (pts.last(), pts.first()) {
                painter.line_segment([last_pt, first_pt], Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 200, 255, 100)));
            }
        }
        for (i, &pt) in pts.iter().enumerate() {
            let color = if i == 0 {
                Color32::from_rgb(0, 255, 100)
            } else {
                Color32::from_rgb(100, 200, 255)
            };
            painter.circle(pt, 4.0, color, Stroke::NONE);
        }
    }

    if app.is_drawing_polygon
        && response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
        && sprite_rect.contains(pos)
    {
        let local_x = ((pos.x - sprite_rect.min.x) / scale).round();
        let local_y = ((pos.y - sprite_rect.min.y) / scale).round();

        let close_threshold = 8.0 / scale;
        // Clicking near the first vertex closes the polygon.
        let should_close =
            geometry::polygon_vertex_hit_test(&app.polygon_in_progress, local_x, local_y, close_threshold).is_some_and(|idx| idx == 0 && app.polygon_in_progress.len() >= 3);

        if should_close {
            let completed = app.polygon_in_progress.clone();
            // The closing edge connects the last vertex back to the first vertex.
            // Reject the close if the resulting polygon would be self-intersecting.
            if geometry::polygon_is_self_intersecting(&completed) {
                app.show_invalid_polygon_placement_modal = true;
            } else {
                app.polygon_in_progress.clear();
                app.is_drawing_polygon = false;
                let polygon_set = if let Some(p) = app.sprite_properties.get_mut(sprite_name) {
                    p.hitbox = HitboxShape::Polygon { points: completed };
                    p.hitbox_enabled = true;
                    true
                } else {
                    false
                };
                if polygon_set {
                    app.has_unsaved_changes = true;
                }
            }
        } else if geometry::polyline_would_self_intersect_adding_vertex(&app.polygon_in_progress, [local_x, local_y]) {
            app.show_invalid_polygon_placement_modal = true;
        } else {
            app.polygon_in_progress.push([local_x, local_y]);
        }
        ctx.request_repaint();
    }

    if app.is_drawing_polygon && response.hovered() {
        ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
    }
}

fn handle_pivot_drag(app: &mut ZephyrApp, response: &egui::Response, sprite_name: &str, sprite_rect: Rect, scale: f32, sprite_w: f32, sprite_h: f32, ctx: &egui::Context) {
    if !app.sprite_properties.get(sprite_name).is_some_and(|p| p.pivot_enabled) {
        app.pivot_drag = false;
        return;
    }
    let (dw, dh) = (sprite_w * scale, sprite_h * scale);
    let near_pivot = |pos: Pos2, props: &SpriteProperties| -> bool {
        let d = pos - Pos2::new(sprite_rect.min.x + props.pivot.x * dw, sprite_rect.min.y + props.pivot.y * dh);
        d.length_sq() <= 64.0
    };
    if response.drag_started_by(egui::PointerButton::Primary) && !app.pivot_drag {
        let hit = ctx
            .input(|i| i.pointer.press_origin())
            .or_else(|| response.interact_pointer_pos())
            .and_then(|pos| app.sprite_properties.get(sprite_name).map(|p| near_pivot(pos, p)))
            .unwrap_or(false);
        app.pivot_drag = hit;
        if hit {
            if let Some(props) = app.sprite_properties.get(sprite_name) {
                app.pivot_drag_start = [props.pivot.x, props.pivot.y];
            }
            app.pivot_drag_delta = egui::Vec2::ZERO;
        }
    }
    if response.dragged_by(egui::PointerButton::Primary) && app.pivot_drag {
        let delta = response.drag_delta();
        // Accumulate sprite-pixel delta from origin so snapping doesn't discard
        // sub-pixel movement between frames.
        app.pivot_drag_delta += egui::vec2(delta.x / scale, delta.y / scale);
        let [start_x, start_y] = app.pivot_drag_start;
        let raw_px = start_x * sprite_w + app.pivot_drag_delta.x;
        let raw_py = start_y * sprite_h + app.pivot_drag_delta.y;
        let (final_px, final_py) = (raw_px.round(), raw_py.round());
        if let Some(props) = app.sprite_properties.get_mut(sprite_name) {
            props.pivot.x = (final_px / sprite_w).clamp(0.0, 1.0);
            props.pivot.y = (final_py / sprite_h).clamp(0.0, 1.0);
            if props.pivot.preset != PivotPreset::Custom {
                props.pivot.preset = PivotPreset::Custom;
            }
        }
        ctx.request_repaint();
    }
    if !response.dragged_by(egui::PointerButton::Primary) {
        let was_dragging = app.pivot_drag;
        app.pivot_drag = false;
        if was_dragging {
            app.has_unsaved_changes = true;
        }
    }
}

fn find_drag_target(hitbox: &HitboxShape, lx: f32, ly: f32, scale: f32) -> Option<HitboxDragTarget> {
    let threshold = 8.0 / scale;
    match hitbox {
        HitboxShape::Rectangle { x, y, w, h } => {
            if let Some(handle) = geometry::rect_handle_hit_test(*x, *y, *w, *h, lx, ly, threshold) {
                return Some(HitboxDragTarget::RectHandle(handle));
            }
            // Body: point inside the rectangle.
            if lx >= *x && lx <= x + w && ly >= *y && ly <= y + h {
                return Some(HitboxDragTarget::Body);
            }
            None
        }
        HitboxShape::Circle { cx, cy, radius } => {
            if let Some(handle) = geometry::circle_handle_hit_test(*cx, *cy, *radius, lx, ly, threshold) {
                return Some(HitboxDragTarget::CircleHandle(handle));
            }
            if geometry::circle_hit_test(*cx, *cy, *radius, lx, ly, threshold).is_some() {
                return Some(HitboxDragTarget::Body);
            }
            None
        }
        HitboxShape::Polygon { points } => {
            if let Some(idx) = geometry::polygon_vertex_hit_test(points, lx, ly, threshold) {
                return Some(HitboxDragTarget::PolygonVertex(idx));
            }
            if geometry::point_in_polygon(lx, ly, points) {
                return Some(HitboxDragTarget::Body);
            }
            None
        }
    }
}

fn apply_hitbox_drag(original: &HitboxShape, target: HitboxDragTarget, dx: f32, dy: f32) -> HitboxShape {
    match (original, target) {
        (HitboxShape::Rectangle { x, y, w, h }, HitboxDragTarget::RectHandle(handle)) => {
            let (mut nx, mut ny, mut nw, mut nh) = (*x, *y, *w, *h);
            match handle {
                RectHandle::TopLeft => {
                    nx += dx;
                    ny += dy;
                    nw -= dx;
                    nh -= dy;
                }
                RectHandle::TopCenter => {
                    ny += dy;
                    nh -= dy;
                }
                RectHandle::TopRight => {
                    ny += dy;
                    nw += dx;
                    nh -= dy;
                }
                RectHandle::MiddleLeft => {
                    nx += dx;
                    nw -= dx;
                }
                RectHandle::MiddleRight => {
                    nw += dx;
                }
                RectHandle::BottomLeft => {
                    nx += dx;
                    nw -= dx;
                    nh += dy;
                }
                RectHandle::BottomCenter => {
                    nh += dy;
                }
                RectHandle::BottomRight => {
                    nw += dx;
                    nh += dy;
                }
            }
            HitboxShape::Rectangle {
                x: nx,
                y: ny,
                w: nw.max(1.0),
                h: nh.max(1.0),
            }
        }
        (HitboxShape::Rectangle { x, y, w, h }, HitboxDragTarget::Body) => HitboxShape::Rectangle { x: x + dx, y: y + dy, w: *w, h: *h },
        (HitboxShape::Circle { cx, cy, radius }, HitboxDragTarget::CircleHandle(handle)) => {
            let new_radius = match handle {
                CircleHandle::North => radius - dy,
                CircleHandle::East => radius + dx,
                CircleHandle::South => radius + dy,
                CircleHandle::West => radius - dx,
            };
            HitboxShape::Circle {
                cx: *cx,
                cy: *cy,
                radius: new_radius.max(1.0),
            }
        }
        (HitboxShape::Circle { cx, cy, radius }, HitboxDragTarget::Body) => HitboxShape::Circle {
            cx: cx + dx,
            cy: cy + dy,
            radius: *radius,
        },
        (HitboxShape::Polygon { points }, HitboxDragTarget::PolygonVertex(idx)) => {
            let mut new_pts = points.clone();
            if let Some(pt) = new_pts.get_mut(idx) {
                pt[0] += dx;
                pt[1] += dy;
            }
            HitboxShape::Polygon { points: new_pts }
        }
        (HitboxShape::Polygon { points }, HitboxDragTarget::Body) => {
            let new_pts = points.iter().map(|&[px, py]| [px + dx, py + dy]).collect();
            HitboxShape::Polygon { points: new_pts }
        }
        // Mismatched shape/target (shouldn't happen at runtime): return original unchanged.
        _ => original.clone(),
    }
}

fn snap_hitbox(shape: HitboxShape) -> HitboxShape {
    match shape {
        HitboxShape::Rectangle { x, y, w, h } => HitboxShape::Rectangle {
            x: x.round(),
            y: y.round(),
            w: w.round().max(1.0),
            h: h.round().max(1.0),
        },
        HitboxShape::Circle { cx, cy, radius } => HitboxShape::Circle {
            cx: cx.round(),
            cy: cy.round(),
            radius: radius.round().max(1.0),
        },
        HitboxShape::Polygon { points } => HitboxShape::Polygon {
            points: points.into_iter().map(|[x, y]| [x.round(), y.round()]).collect(),
        },
    }
}

fn clamp_hitbox(shape: HitboxShape, sprite_w: f32, sprite_h: f32) -> HitboxShape {
    match shape {
        HitboxShape::Rectangle { x, y, w, h } => {
            let (cx, cy, cw, ch) = geometry::clamp_rect_to_bounds(x, y, w, h, sprite_w, sprite_h);
            HitboxShape::Rectangle { x: cx, y: cy, w: cw, h: ch }
        }
        HitboxShape::Circle { cx, cy, radius } => {
            let (ncx, ncy, nr) = geometry::clamp_circle_to_bounds(cx, cy, radius, sprite_w, sprite_h);
            HitboxShape::Circle { cx: ncx, cy: ncy, radius: nr }
        }
        HitboxShape::Polygon { points } => HitboxShape::Polygon {
            points: geometry::clamp_polygon_to_bounds(&points, sprite_w, sprite_h),
        },
    }
}

fn handle_hitbox_drag(app: &mut ZephyrApp, response: &egui::Response, sprite_name: &str, sprite_rect: Rect, scale: f32, sprite_w: f32, sprite_h: f32, ctx: &egui::Context) {
    if response.drag_started_by(egui::PointerButton::Primary) {
        let start_pos = ctx.input(|i| i.pointer.press_origin()).or_else(|| response.interact_pointer_pos());
        if let Some(pos) = start_pos {
            let lx = (pos.x - sprite_rect.min.x) / scale;
            let ly = (pos.y - sprite_rect.min.y) / scale;

            // Hit-test to find what to drag; bail if the user clicked empty space.
            let maybe_target = app
                .sprite_properties
                .get(sprite_name)
                .filter(|p| p.hitbox_enabled)
                .and_then(|p| find_drag_target(&p.hitbox, lx, ly, scale));

            if let Some(target) = maybe_target {
                if let Some(props) = app.sprite_properties.get(sprite_name) {
                    let original = props.hitbox.clone();
                    app.hitbox_drag = Some(HitboxDragState {
                        target,
                        original,
                        accumulated_dx: 0.0,
                        accumulated_dy: 0.0,
                    });
                }
            }
        }
    }

    if response.dragged_by(egui::PointerButton::Primary) {
        // Accumulate the total sprite-pixel delta from drag start so snapping
        // consumes sub-pixel movement rather than discarding it each frame.
        let delta = response.drag_delta();
        if let Some(drag_state) = app.hitbox_drag.as_mut() {
            drag_state.accumulated_dx += delta.x / scale;
            drag_state.accumulated_dy += delta.y / scale;
        }
        let maybe_drag = app.hitbox_drag.as_ref().map(|d| (d.target, d.original.clone(), d.accumulated_dx, d.accumulated_dy));
        if let Some((target, original, adx, ady)) = maybe_drag {
            if let Some(props) = app.sprite_properties.get_mut(sprite_name) {
                // For a polygon body drag the whole shape moves as a rigid body.
                // Pre-clamp the total accumulated delta so the polygon stops at the
                // boundary rather than individual vertices deforming against it.
                let (cdx, cdy) = if target == HitboxDragTarget::Body {
                    if let HitboxShape::Polygon { points } = &original {
                        geometry::clamp_polygon_translation(points, adx, ady, sprite_w, sprite_h)
                    } else {
                        (adx, ady)
                    }
                } else {
                    (adx, ady)
                };
                let dragged = apply_hitbox_drag(&original, target, cdx, cdy);
                let clamped = clamp_hitbox(dragged, sprite_w, sprite_h);
                let final_shape = snap_hitbox(clamped);
                // Reject polygon vertex drags that would create a self-intersection;
                // the vertex stops at the last valid position instead.
                let valid = match (&final_shape, target) {
                    (HitboxShape::Polygon { points }, HitboxDragTarget::PolygonVertex(_)) => !geometry::polygon_is_self_intersecting(points),
                    _ => true,
                };
                if valid {
                    props.hitbox = final_shape;
                }
            }
        }
    }

    // Clear drag state the moment the primary button is released.
    if !response.dragged_by(egui::PointerButton::Primary) {
        let had_drag = app.hitbox_drag.is_some();
        app.hitbox_drag = None;
        if had_drag {
            app.has_unsaved_changes = true;
        }
    }
}

fn handle_color(highlighted: bool) -> Color32 {
    if highlighted { Color32::from_rgb(255, 220, 0) } else { Color32::WHITE }
}

fn draw_handle_square(painter: &egui::Painter, sx: f32, sy: f32, color: Color32) {
    const HALF: f32 = 4.0;
    let r = Rect::from_min_max(Pos2::new(sx - HALF, sy - HALF), Pos2::new(sx + HALF, sy + HALF));
    painter.rect(r, 0.0, color, Stroke::new(1.0, Color32::from_rgb(30, 30, 30)), egui::StrokeKind::Outside);
}

fn draw_hitbox_overlay(ui: &mut Ui, sprite_rect: &Rect, clip_rect: &Rect, hitbox: &HitboxShape, scale: f32, drag: &Option<HitboxDragState>, hover: Option<HitboxDragTarget>) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(255, 80, 80));
    let painter = ui.painter().with_clip_rect(*clip_rect);

    // Is the given target currently highlighted (hovered or being dragged)?
    let is_lit = |target: HitboxDragTarget| -> bool { drag.as_ref().is_some_and(|d| d.target == target) || hover.is_some_and(|h| h == target) };
    let body_lit = is_lit(HitboxDragTarget::Body);

    // Body fill brightens when hovered or dragged to signal it's moveable.
    let fill = if body_lit {
        Color32::from_rgba_unmultiplied(255, 120, 50, 75)
    } else {
        Color32::from_rgba_unmultiplied(255, 80, 80, 30)
    };

    match hitbox {
        HitboxShape::Rectangle { x, y, w, h } => {
            let r = Rect::from_min_size(Pos2::new(sprite_rect.min.x + x * scale, sprite_rect.min.y + y * scale), vec2(w * scale, h * scale));
            painter.rect(r, 0.0, fill, stroke, egui::StrokeKind::Outside);

            let cx_s = sprite_rect.min.x + (x + w * 0.5) * scale;
            let cy_s = sprite_rect.min.y + (y + h * 0.5) * scale;
            let left_s = sprite_rect.min.x + x * scale;
            let right_s = sprite_rect.min.x + (x + w) * scale;
            let top_s = sprite_rect.min.y + y * scale;
            let bot_s = sprite_rect.min.y + (y + h) * scale;

            let handle_positions: &[(f32, f32, RectHandle)] = &[
                (left_s, top_s, RectHandle::TopLeft),
                (cx_s, top_s, RectHandle::TopCenter),
                (right_s, top_s, RectHandle::TopRight),
                (left_s, cy_s, RectHandle::MiddleLeft),
                (right_s, cy_s, RectHandle::MiddleRight),
                (left_s, bot_s, RectHandle::BottomLeft),
                (cx_s, bot_s, RectHandle::BottomCenter),
                (right_s, bot_s, RectHandle::BottomRight),
            ];
            for &(sx, sy, handle) in handle_positions {
                let lit = body_lit || is_lit(HitboxDragTarget::RectHandle(handle));
                draw_handle_square(&painter, sx, sy, handle_color(lit));
            }
        }
        HitboxShape::Circle { cx, cy, radius } => {
            let centre = Pos2::new(sprite_rect.min.x + cx * scale, sprite_rect.min.y + cy * scale);
            painter.circle(centre, radius * scale, fill, stroke);

            let r_s = radius * scale;
            let cardinal: &[(f32, f32, CircleHandle)] = &[
                (centre.x, centre.y - r_s, CircleHandle::North),
                (centre.x + r_s, centre.y, CircleHandle::East),
                (centre.x, centre.y + r_s, CircleHandle::South),
                (centre.x - r_s, centre.y, CircleHandle::West),
            ];
            for &(sx, sy, handle) in cardinal {
                let lit = body_lit || is_lit(HitboxDragTarget::CircleHandle(handle));
                draw_handle_square(&painter, sx, sy, handle_color(lit));
            }
        }
        HitboxShape::Polygon { points } if points.len() >= 2 => {
            let pts: Vec<Pos2> = points
                .iter()
                .map(|&[px, py]| Pos2::new(sprite_rect.min.x + px * scale, sprite_rect.min.y + py * scale))
                .collect();

            let self_intersecting = geometry::polygon_is_self_intersecting(points);

            // Orange outline when the polygon is self-intersecting (invalid state);
            // normal red otherwise.
            let poly_stroke = if self_intersecting {
                Stroke::new(1.5, Color32::from_rgb(255, 165, 0))
            } else {
                stroke
            };

            // Only fill valid (non-self-intersecting) polygons.
            if !self_intersecting {
                for [ai, bi, ci] in geometry::ear_clip_triangulate(points) {
                    if let (Some(&pa), Some(&pb), Some(&pc)) = (pts.get(ai), pts.get(bi), pts.get(ci)) {
                        painter.add(egui::Shape::convex_polygon(vec![pa, pb, pc], fill, Stroke::NONE));
                    }
                }
            }

            // Draw outline edges; windows handles sequential pairs, last edge closes the polygon.
            for window in pts.windows(2) {
                if let [a, b] = window {
                    painter.line_segment([*a, *b], poly_stroke);
                }
            }
            if let (Some(&last_pt), Some(&first_pt)) = (pts.last(), pts.first()) {
                painter.line_segment([last_pt, first_pt], poly_stroke);
            }

            // Draw vertex handles (larger than before to read as interactive).
            for (i, &pt) in pts.iter().enumerate() {
                let lit = body_lit || is_lit(HitboxDragTarget::PolygonVertex(i));
                painter.circle(pt, 5.0, handle_color(lit), Stroke::new(1.0, Color32::from_rgb(30, 30, 30)));
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HitboxShape;

    #[test]
    fn snap_hitbox_rounds_rect_fields() {
        let snapped = snap_hitbox(HitboxShape::Rectangle { x: 1.3, y: 2.7, w: 10.6, h: 5.4 });
        assert!(matches!(snapped, HitboxShape::Rectangle { x, y, w, h } if x == 1.0 && y == 3.0 && w == 11.0 && h == 5.0));
    }

    #[test]
    fn snap_hitbox_enforces_min_one_on_rect_size() {
        let snapped = snap_hitbox(HitboxShape::Rectangle { x: 0.0, y: 0.0, w: 0.3, h: 0.2 });
        assert!(matches!(snapped, HitboxShape::Rectangle { w, h, .. } if w == 1.0 && h == 1.0));
    }

    #[test]
    fn snap_hitbox_rounds_circle_fields() {
        let snapped = snap_hitbox(HitboxShape::Circle { cx: 3.6, cy: 1.2, radius: 7.8 });
        assert!(matches!(snapped, HitboxShape::Circle { cx, cy, radius } if cx == 4.0 && cy == 1.0 && radius == 8.0));
    }

    #[test]
    fn snap_hitbox_rounds_polygon_vertices() {
        let snapped = snap_hitbox(HitboxShape::Polygon {
            points: vec![[0.4, 0.6], [5.5, 2.3], [3.1, 8.9]],
        });
        if let HitboxShape::Polygon { points } = snapped {
            assert_eq!(points, vec![[0.0, 1.0], [6.0, 2.0], [3.0, 9.0]]);
        } else {
            panic!("expected Polygon variant");
        }
    }
}
