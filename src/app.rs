// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

mod animation_list_panel;
mod animation_panel;
mod atlas_settings_panel;
mod chrome;
mod file_ops;
mod preview_panel;
mod settings_panel;
mod sprite_panel;
mod sprite_properties_panel;
mod state;
mod thumbnails;
mod tree_panel;
pub(crate) use state::{AppTab, FrameDragPayload, SpriteDragPayload, ZephyrApp};

impl eframe::App for ZephyrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);
        self.ensure_atlas_texture(ctx);

        // Modals
        tree_panel::render_create_directory_modal(self, ctx);
        tree_panel::render_clear_confirmation_modal(self, ctx);
        tree_panel::render_rename_modal(self, ctx);
        animation_list_panel::render_animation_rename_modal(self, ctx);
        animation_list_panel::render_new_animation_modal(self, ctx);

        // Panels (order matters: side panels before central panel)
        chrome::render_menu_bar(self, ctx);
        let zoom_action = chrome::render_status_bar(self, ctx);

        match (zoom_action, self.current_tab) {
            (chrome::ZoomAction::ZoomIn, state::AppTab::Preview) => self.zoom_level = (self.zoom_level * 1.2).min(10.0),
            (chrome::ZoomAction::ZoomIn, state::AppTab::SpriteEditor) => self.sprite_editor_zoom = (self.sprite_editor_zoom * 1.2).min(20.0),
            (chrome::ZoomAction::ZoomIn, state::AppTab::AnimationEditor) => self.anim_preview_zoom = (self.anim_preview_zoom * 1.2).min(20.0),
            (chrome::ZoomAction::ZoomOut, state::AppTab::Preview) => self.zoom_level = (self.zoom_level / 1.2).max(0.1),
            (chrome::ZoomAction::ZoomOut, state::AppTab::SpriteEditor) => self.sprite_editor_zoom = (self.sprite_editor_zoom / 1.2).max(0.1),
            (chrome::ZoomAction::ZoomOut, state::AppTab::AnimationEditor) => self.anim_preview_zoom = (self.anim_preview_zoom / 1.2).max(0.1),
            (chrome::ZoomAction::Reset, state::AppTab::Preview) => {
                self.zoom_level = 1.0;
                self.pan_offset = egui::Vec2::ZERO;
            }
            (chrome::ZoomAction::Reset, state::AppTab::SpriteEditor) => {
                self.sprite_editor_zoom = 1.0;
                self.sprite_editor_pan = egui::Vec2::ZERO;
            }
            (chrome::ZoomAction::Reset, state::AppTab::AnimationEditor) => {
                self.anim_preview_zoom = 1.0;
                self.anim_preview_pan = egui::Vec2::ZERO;
            }
            (chrome::ZoomAction::None, _) => {}
        }

        tree_panel::render(self, ctx);
        settings_panel::render(self, ctx);

        // Upload atlas pixels to the GPU if a pack is pending before rendering
        // the central panel.
        self.ensure_atlas_texture(ctx);

        chrome::render_central_panel(self, ctx);

        if self.has_unsaved_changes && self.project_path.is_some() {
            file_ops::auto_save(self);
        }
    }
}

fn draw_checker(ui: &mut egui::Ui, rect: &egui::Rect, clip_rect: &egui::Rect, tile_px: f32) {
    let cols = (rect.width() / tile_px).ceil() as i32;
    let rows = (rect.height() / tile_px).ceil() as i32;
    let c1 = egui::Color32::from_rgb(0x80, 0x80, 0x80);
    let c2 = egui::Color32::from_rgb(0xc0, 0xc0, 0xc0);
    let painter = ui.painter().with_clip_rect(*clip_rect);
    for row in 0..rows {
        for col in 0..cols {
            let color = if (row + col) % 2 == 0 { c1 } else { c2 };
            let cell = egui::Rect::from_min_size(
                egui::Pos2::new(rect.min.x + col as f32 * tile_px, rect.min.y + row as f32 * tile_px),
                egui::vec2(tile_px, tile_px),
            );
            painter.rect_filled(cell.intersect(*rect), 0.0, color);
        }
    }
}
