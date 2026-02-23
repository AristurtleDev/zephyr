// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use egui::{Color32, Pos2, Rect, Ui, vec2};
use std::collections::HashSet;

use super::state::BackgroundStyle;
use super::{AppTab, ZephyrApp};
use crate::types::{NodeId, TreeNode};

pub(super) fn render_content(app: &mut ZephyrApp, ui: &mut Ui) {
    render_preview(app, ui);
}

fn render_preview(app: &mut ZephyrApp, ui: &mut Ui) {
    let mut click_data: Option<(Pos2, Rect, egui::Vec2, u32, u32)> = None;
    let mut double_click_data: Option<(Pos2, Rect, egui::Vec2, u32, u32)> = None;
    let mut drag_rect_data: Option<(Rect, Rect, egui::Vec2, u32, u32)> = None;

    if let Some(atlas) = &app.atlas {
        if let Some(texture) = &app.atlas_texture {
            let available_size = ui.available_size();

            let atlas_aspect = atlas.width as f32 / atlas.height as f32;
            let available_aspect = available_size.x / available_size.y;

            let base_size = if atlas_aspect > available_aspect {
                let width = available_size.x * 0.9;
                let height = width / atlas_aspect;
                vec2(width, height)
            } else {
                let height = available_size.y * 0.9;
                let width = height * atlas_aspect;
                vec2(width, height)
            };

            let (full_rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

            if response.hovered() {
                let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                if scroll_delta.abs() > 0.0 {
                    let old_zoom = app.zoom_level;
                    let zoom_factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                    app.zoom_level = (app.zoom_level * zoom_factor).clamp(0.1, 10.0);

                    // Zoom towards cursor
                    if let Some(mouse) = response.hover_pos() {
                        let center = full_rect.center() + app.pan_offset;
                        let mouse_to_center = mouse - center;
                        let zoom_ratio = app.zoom_level / old_zoom;
                        app.pan_offset += mouse_to_center * (1.0 - zoom_ratio);
                    }
                }
            }

            if response.dragged_by(egui::PointerButton::Middle) {
                app.pan_offset += response.drag_delta();
            }

            let display_size = base_size * app.zoom_level;

            let center_x = full_rect.center().x + app.pan_offset.x;
            let center_y = full_rect.center().y + app.pan_offset.y;
            let image_rect = Rect::from_center_size(Pos2::new(center_x, center_y), display_size);

            let clip_rect = full_rect.intersect(ui.clip_rect());

            ui.painter().rect_filled(full_rect, 0.0, Color32::from_rgb(64, 64, 64));

            match app.prefs.background_style {
                BackgroundStyle::Checkerboard => {
                    let tile_px = 32.0 * display_size.x / atlas.width as f32;
                    super::draw_checker(ui, &image_rect, &clip_rect, tile_px);
                }
                BackgroundStyle::SolidColor => {
                    ui.painter().with_clip_rect(clip_rect).rect_filled(image_rect, 0.0, app.prefs.solid_bg_color);
                }
                BackgroundStyle::Off => {}
            }

            let selected_image_ids = collect_selected_image_ids(&app.tree_root, &app.selected_nodes);

            // Pre-compute which placement names are selected so opacity rendering
            // and selection boxes both use the same set without repeating the tree walk.
            let selected_names: HashSet<String> = if selected_image_ids.is_empty() {
                HashSet::new()
            } else {
                atlas
                    .placements
                    .iter()
                    .filter_map(|p| {
                        app.tree_root
                            .find_image_id_by_name(&p.name)
                            .filter(|id| selected_image_ids.contains(id))
                            .map(|_| p.name.clone())
                    })
                    .collect()
            };

            draw_atlas_with_opacity(ui, texture, &image_rect, &clip_rect, atlas, &selected_names, display_size);

            if app.prefs.draw_border {
                ui.painter()
                    .with_clip_rect(clip_rect)
                    .rect_stroke(image_rect, 0.0, (1.0, Color32::WHITE), egui::StrokeKind::Outside);
            }

            draw_selection_boxes(atlas, ui, &image_rect, &clip_rect, display_size, &selected_names);

            if response.drag_started_by(egui::PointerButton::Primary)
                && let Some(start_pos) = response.interact_pointer_pos()
            {
                app.drag_select_start = Some(start_pos);
                app.drag_select_current = Some(start_pos);
            }

            if response.dragged_by(egui::PointerButton::Primary)
                && let Some(current_pos) = response.interact_pointer_pos()
            {
                app.drag_select_current = Some(current_pos);
            }

            if response.drag_stopped_by(egui::PointerButton::Primary) {
                if let (Some(start), Some(end)) = (app.drag_select_start, app.drag_select_current) {
                    drag_rect_data = Some((Rect::from_two_pos(start, end), image_rect, display_size, atlas.width, atlas.height));
                }
                app.drag_select_start = None;
                app.drag_select_current = None;
            }

            if let (Some(start), Some(current)) = (app.drag_select_start, app.drag_select_current) {
                let drag_rect = Rect::from_two_pos(start, current);
                ui.painter()
                    .rect_stroke(drag_rect, 0.0, (2.0, Color32::from_rgb(100, 150, 255)), egui::StrokeKind::Outside);
                ui.painter().rect_filled(drag_rect, 0.0, Color32::from_rgba_unmultiplied(100, 150, 255, 50));
            }

            // Store click data for processing after the atlas borrow ends.
            if response.clicked()
                && !response.dragged()
                && let Some(click_pos) = response.interact_pointer_pos()
            {
                click_data = Some((click_pos, image_rect, display_size, atlas.width, atlas.height));
            }

            if response.double_clicked()
                && let Some(click_pos) = response.interact_pointer_pos()
            {
                double_click_data = Some((click_pos, image_rect, display_size, atlas.width, atlas.height));
            }
        }
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.label("No atlas generated yet.");
            ui.label("Add images to generate an atlas.");
        });
    }

    // Handle click after the atlas borrow has ended
    if let Some((click_pos, image_rect, display_size, atlas_width, atlas_height)) = click_data {
        handle_sprite_click(app, click_pos, &image_rect, display_size, atlas_width, atlas_height, ui);
    }

    if let Some((click_pos, image_rect, display_size, atlas_width, atlas_height)) = double_click_data {
        handle_sprite_double_click(app, click_pos, &image_rect, display_size, atlas_width, atlas_height);
    }

    // Handle drag selection after the atlas borrow has ended
    if let Some((drag_rect, image_rect, display_size, atlas_width, atlas_height)) = drag_rect_data {
        handle_drag_selection(app, drag_rect, &image_rect, display_size, atlas_width, atlas_height, ui);
    }
}

fn collect_selected_image_ids(tree: &TreeNode, selected_nodes: &HashSet<NodeId>) -> HashSet<NodeId> {
    let mut result = HashSet::new();

    fn collect_from_node(node: &TreeNode, selected: &HashSet<NodeId>, result: &mut HashSet<NodeId>) {
        match node {
            TreeNode::Directory(dir) => {
                if selected.contains(&dir.id) {
                    collect_all_images(node, result);
                } else {
                    for child in &dir.children {
                        collect_from_node(child, selected, result);
                    }
                }
            }
            TreeNode::Image(img) => {
                if selected.contains(&img.id) {
                    result.insert(img.id);
                }
            }
        }
    }

    fn collect_all_images(node: &TreeNode, result: &mut HashSet<NodeId>) {
        match node {
            TreeNode::Directory(dir) => {
                for child in &dir.children {
                    collect_all_images(child, result);
                }
            }
            TreeNode::Image(img) => {
                result.insert(img.id);
            }
        }
    }

    collect_from_node(tree, selected_nodes, &mut result);
    result
}

fn draw_atlas_with_opacity(
    ui: &mut Ui,
    texture: &egui::TextureHandle,
    image_rect: &Rect,
    clip_rect: &Rect,
    atlas: &crate::types::PackedAtlas,
    selected_names: &HashSet<String>,
    display_size: egui::Vec2,
) {
    if selected_names.is_empty() {
        ui.painter()
            .with_clip_rect(*clip_rect)
            .image(texture.id(), *image_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        return;
    }

    let display_pixels_per_atlas_pixel = display_size.x / atlas.width as f32;
    let image_min = image_rect.min;
    let atlas_width = atlas.width as f32;
    let atlas_height = atlas.height as f32;

    ui.painter().with_clip_rect(*clip_rect).image(
        texture.id(),
        *image_rect,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::from_rgba_unmultiplied(255, 255, 255, 128),
    );

    for placement in &atlas.placements {
        if selected_names.contains(&placement.name) {
            let u_min = placement.x as f32 / atlas_width;
            let v_min = placement.y as f32 / atlas_height;
            let u_max = (placement.x + placement.width) as f32 / atlas_width;
            let v_max = (placement.y + placement.height) as f32 / atlas_height;

            let x = image_min.x + (placement.x as f32 * display_pixels_per_atlas_pixel);
            let y = image_min.y + (placement.y as f32 * display_pixels_per_atlas_pixel);
            let w = placement.width as f32 * display_pixels_per_atlas_pixel;
            let h = placement.height as f32 * display_pixels_per_atlas_pixel;

            let sprite_rect = Rect::from_min_size(Pos2::new(x, y), vec2(w, h));
            let uv_rect = Rect::from_min_max(Pos2::new(u_min, v_min), Pos2::new(u_max, v_max));

            ui.painter().with_clip_rect(*clip_rect).image(texture.id(), sprite_rect, uv_rect, Color32::WHITE);
        }
    }
}

fn handle_sprite_double_click(app: &mut ZephyrApp, click_pos: Pos2, image_rect: &Rect, display_size: egui::Vec2, atlas_width: u32, atlas_height: u32) {
    let display_pixels_per_atlas_pixel = display_size.x / atlas_width as f32;
    let image_min = image_rect.min;

    let atlas_x = ((click_pos.x - image_min.x) / display_pixels_per_atlas_pixel) as u32;
    let atlas_y = ((click_pos.y - image_min.y) / display_pixels_per_atlas_pixel) as u32;

    if atlas_x >= atlas_width || atlas_y >= atlas_height {
        return;
    }

    if let Some(atlas) = &app.atlas {
        for placement in &atlas.placements {
            if atlas_x >= placement.x
                && atlas_x < placement.x + placement.width
                && atlas_y >= placement.y
                && atlas_y < placement.y + placement.height
                && let Some(id) = app.tree_root.find_image_id_by_name(&placement.name)
            {
                app.selected_nodes.clear();
                app.selected_nodes.insert(id);
                app.current_tab = AppTab::SpriteEditor;
                break;
            }
        }
    }
}

fn handle_sprite_click(app: &mut ZephyrApp, click_pos: Pos2, image_rect: &Rect, display_size: egui::Vec2, atlas_width: u32, atlas_height: u32, ui: &mut Ui) {
    let display_pixels_per_atlas_pixel = display_size.x / atlas_width as f32;
    let image_min = image_rect.min;

    let atlas_x = ((click_pos.x - image_min.x) / display_pixels_per_atlas_pixel) as u32;
    let atlas_y = ((click_pos.y - image_min.y) / display_pixels_per_atlas_pixel) as u32;

    if atlas_x >= atlas_width || atlas_y >= atlas_height {
        return;
    }

    let mut clicked_id: Option<NodeId> = None;

    if let Some(atlas) = &app.atlas {
        for placement in &atlas.placements {
            if atlas_x >= placement.x && atlas_x < placement.x + placement.width && atlas_y >= placement.y && atlas_y < placement.y + placement.height {
                if let Some(id) = app.tree_root.find_image_id_by_name(&placement.name) {
                    clicked_id = Some(id);
                    break;
                }
            }
        }
    }

    let multi_select = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);

    if let Some(id) = clicked_id {
        if multi_select {
            if app.selected_nodes.contains(&id) {
                app.selected_nodes.remove(&id);
            } else {
                app.selected_nodes.insert(id);
            }
        } else {
            app.selected_nodes.clear();
            app.selected_nodes.insert(id);
        }
    } else if !multi_select {
        app.selected_nodes.clear();
    }
}

fn handle_drag_selection(app: &mut ZephyrApp, drag_rect: Rect, image_rect: &Rect, display_size: egui::Vec2, atlas_width: u32, atlas_height: u32, ui: &mut Ui) {
    let display_pixels_per_atlas_pixel = display_size.x / atlas_width as f32;
    let image_min = image_rect.min;

    let atlas_min_x = ((drag_rect.min.x - image_min.x) / display_pixels_per_atlas_pixel).max(0.0) as u32;
    let atlas_min_y = ((drag_rect.min.y - image_min.y) / display_pixels_per_atlas_pixel).max(0.0) as u32;
    let atlas_max_x = ((drag_rect.max.x - image_min.x) / display_pixels_per_atlas_pixel).min(atlas_width as f32) as u32;
    let atlas_max_y = ((drag_rect.max.y - image_min.y) / display_pixels_per_atlas_pixel).min(atlas_height as f32) as u32;

    let selection_rect_atlas = Rect::from_min_max(Pos2::new(atlas_min_x as f32, atlas_min_y as f32), Pos2::new(atlas_max_x as f32, atlas_max_y as f32));

    let mut selected_ids = HashSet::new();

    if let Some(atlas) = &app.atlas {
        for placement in &atlas.placements {
            let sprite_rect = Rect::from_min_max(
                Pos2::new(placement.x as f32, placement.y as f32),
                Pos2::new((placement.x + placement.width) as f32, (placement.y + placement.height) as f32),
            );

            if sprite_rect.intersects(selection_rect_atlas)
                && let Some(id) = app.tree_root.find_image_id_by_name(&placement.name)
            {
                selected_ids.insert(id);
            }
        }
    }

    let multi_select = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);

    if multi_select {
        app.selected_nodes.extend(selected_ids);
    } else {
        app.selected_nodes = selected_ids;
    }
}

fn draw_selection_boxes(atlas: &crate::types::PackedAtlas, ui: &mut Ui, image_rect: &Rect, clip_rect: &Rect, display_size: egui::Vec2, selected_names: &HashSet<String>) {
    let display_pixels_per_atlas_pixel = display_size.x / atlas.width as f32;
    let image_min = image_rect.min;

    for placement in &atlas.placements {
        if selected_names.contains(&placement.name) {
            let x = placement.x as f32 * display_pixels_per_atlas_pixel;
            let y = placement.y as f32 * display_pixels_per_atlas_pixel;
            let w = placement.width as f32 * display_pixels_per_atlas_pixel;
            let h = placement.height as f32 * display_pixels_per_atlas_pixel;

            let sprite_rect = Rect::from_min_size(image_min + vec2(x, y), vec2(w, h));

            ui.painter()
                .with_clip_rect(*clip_rect)
                .rect_stroke(sprite_rect, 0.0, (2.0, Color32::from_rgb(0, 255, 0)), egui::StrokeKind::Outside);
        }
    }
}
