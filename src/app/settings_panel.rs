// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use egui_phosphor::regular::{CARET_DOWN, CARET_RIGHT};

use super::{AppTab, ZephyrApp};

pub(super) fn render(app: &mut ZephyrApp, ctx: &egui::Context) {
    egui::SidePanel::right("properties_panel")
        .default_width(250.0)
        .resizable(true)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let panel_rect = ui.max_rect();
            ui.painter().rect_filled(panel_rect, 0.0, ui.style().visuals.panel_fill);
            let margin = ui.style().spacing.window_margin;
            ui.add_space(margin.top as f32);
            ui.horizontal(|ui| {
                ui.add_space(margin.left as f32);
                ui.heading(match app.current_tab {
                    AppTab::Preview => "Atlas Settings",
                    AppTab::SpriteEditor => "Sprite Properties",
                    AppTab::AnimationEditor => "Animations",
                });
            });
            ui.separator();
            ui.add_space(-6.0);
            match app.current_tab {
                AppTab::Preview => super::atlas_settings_panel::render(app, ui, margin),
                AppTab::SpriteEditor => super::sprite_properties_panel::render(app, ui),
                AppTab::AnimationEditor => super::animation_list_panel::render(app, ui),
            }
        });
}

pub(super) fn two_column_row(ui: &mut egui::Ui, label: &str, add_content: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    let half_width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
    ui.horizontal(|ui| {
        let (label_rect, label_response) = ui.allocate_exact_size(egui::vec2(half_width, ui.spacing().interact_size.y), egui::Sense::hover());
        if ui.is_rect_visible(label_rect) {
            ui.painter().text(
                egui::pos2(label_rect.min.x, label_rect.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(14.0),
                ui.visuals().text_color(),
            );
        }
        add_content(ui);
        label_response
    })
    .inner
}

pub(super) fn render_collapsing_section(ui: &mut egui::Ui, title: &str, margin: egui::Margin, content: impl FnOnce(&mut egui::Ui)) {
    let full_width = ui.available_width();
    let id = ui.make_persistent_id(title);
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);

    let header_response = ui
        .horizontal(|ui| {
            ui.set_min_width(full_width);
            ui.set_max_width(full_width);

            let (rect, response) = ui.allocate_exact_size(egui::vec2(full_width, ui.spacing().interact_size.y + 10.0), egui::Sense::click());

            let bg_color = if response.hovered() {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                ui.visuals().widgets.inactive.bg_fill
            };
            ui.painter().rect_filled(rect, 0.0, bg_color);

            let text_color = ui.visuals().text_color();
            let icon = if state.is_open() { CARET_DOWN } else { CARET_RIGHT };

            ui.painter().text(
                rect.left_center() + egui::vec2(4.0, 0.0),
                egui::Align2::LEFT_CENTER,
                icon,
                egui::FontId::proportional(14.0),
                text_color,
            );
            ui.painter().text(
                rect.left_center() + egui::vec2(24.0, 0.0),
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::proportional(14.0),
                text_color,
            );

            response
        })
        .inner;

    if header_response.clicked() {
        state.toggle(ui);
        state.store(ui.ctx());
    }

    if state.is_open() {
        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: margin.left,
                right: margin.right,
                top: margin.top,
                bottom: margin.bottom,
            })
            .show(ui, |ui| {
                content(ui);
            });
    }
}
