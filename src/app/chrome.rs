// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use egui::{Align2, Color32, Pos2, Rect, Ui, vec2};
use egui_phosphor::regular::{MAGNIFYING_GLASS_MINUS, MAGNIFYING_GLASS_PLUS};

use super::state::{BackgroundStyle, ThemePreference};
use super::{AppTab, ZephyrApp};

pub(super) enum ZoomAction {
    None,
    ZoomIn,
    ZoomOut,
    Reset,
}

pub(super) fn render_menu_bar(app: &mut ZephyrApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Project").clicked() {
                    super::file_ops::new_project(app);
                    ui.close();
                }

                if ui.button("Open Project... (Ctrl+O)").clicked() {
                    super::file_ops::load_project(app);
                    ui.close();
                }

                ui.separator();

                if ui.button("Save Project (Ctrl+S)").clicked() {
                    super::file_ops::save_project(app);
                    ui.close();
                }

                if ui.button("Save Project As...").clicked() {
                    super::file_ops::save_project_as(app);
                    ui.close();
                }

                ui.separator();

                if ui.button("Export Atlas... (Ctrl+E)").clicked() {
                    super::file_ops::export_atlas(app);
                    ui.close();
                }

                ui.separator();

                if ui.button("Preferences").clicked() {
                    app.show_preferences_dialog = true;
                    ui.close();
                }

                ui.separator();

                if ui.button("Exit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close();
                }
            });

            ui.menu_button("Images", |ui| {
                if ui.button("Add Images...").clicked() {
                    super::file_ops::add_images(app);
                    ui.close();
                }

                if ui.button("Add Directory...").clicked() {
                    super::file_ops::import_directory(app);
                    ui.close();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    app.show_about_dialog = true;
                    ui.close();
                }
            });
        });
    });

    render_about_dialog(app, ctx);
    render_preferences_dialog(app, ctx);
}

fn render_about_dialog(app: &mut ZephyrApp, ctx: &egui::Context) {
    if !app.show_about_dialog {
        return;
    }

    let mut open = true;

    egui::Window::new("")
        .id(egui::Id::new("about_dialog"))
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(420.0);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if let Some(texture) = &app.mascot_texture {
                    ui.add(egui::Image::new(texture).max_size(egui::vec2(120.0, 120.0)));
                    ui.add_space(12.0);
                }
                ui.vertical(|ui| {
                    ui.heading("Zephyr");
                    ui.label(concat!("Version ", env!("CARGO_PKG_VERSION")));
                    ui.add_space(8.0);
                    ui.label(env!("CARGO_PKG_DESCRIPTION"));
                    ui.add_space(8.0);
                    ui.label(concat!("\u{00a9} 2026 ", env!("CARGO_PKG_AUTHORS")));
                    ui.label(concat!("Licensed under the ", env!("CARGO_PKG_LICENSE"), " License."));
                    ui.add_space(8.0);
                    ui.label("Built with Rust and egui.");
                });
                ui.add_space(8.0);
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            ui.vertical_centered(|ui| {
                ui.label("Special thanks to:");
            });
            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                ui.add_space(8.0);
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.hyperlink_to("Zander", "https://github.com/FrostyH125");
                ui.label(", ");
                ui.hyperlink_to("Mr. Grak", "https://github.com/MrGrak");
                ui.label(", ");
                ui.hyperlink_to("Prime31", "https://github.com/Prime31");
                ui.label(", ");
                ui.hyperlink_to("Vchelaru", "https://github.com/Vchelaru");
                ui.label(", and the rest of the ");
                ui.hyperlink_to("MonoGame Community", "https://monogame.net");
            });
            ui.add_space(8.0);
        });

    if !open {
        app.show_about_dialog = false;
    }
}

fn render_preferences_dialog(app: &mut ZephyrApp, ctx: &egui::Context) {
    if !app.show_preferences_dialog {
        return;
    }

    let mut open = true;
    let mut theme_changed = false;

    egui::Window::new("Preferences")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(360.0);

            ui.label("Theme:");
            ui.horizontal(|ui| {
                theme_changed |= ui.radio_value(&mut app.prefs.theme, ThemePreference::Auto, "Auto").changed();
                theme_changed |= ui.radio_value(&mut app.prefs.theme, ThemePreference::Dark, "Dark").changed();
                theme_changed |= ui.radio_value(&mut app.prefs.theme, ThemePreference::Light, "Light").changed();
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label("Preview Background:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut app.prefs.background_style, BackgroundStyle::Checkerboard, "Checkerboard");
                ui.radio_value(&mut app.prefs.background_style, BackgroundStyle::SolidColor, "Solid Color");
                ui.radio_value(&mut app.prefs.background_style, BackgroundStyle::Off, "None");
            });

            if app.prefs.background_style == BackgroundStyle::SolidColor {
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.color_edit_button_srgba(&mut app.prefs.solid_bg_color);
                });
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            ui.checkbox(&mut app.prefs.draw_border, "Draw preview border");
        });

    if theme_changed {
        match app.prefs.theme {
            ThemePreference::Auto => ctx.set_visuals(egui::Visuals::default()),
            ThemePreference::Dark => ctx.set_visuals(egui::Visuals::dark()),
            ThemePreference::Light => ctx.set_visuals(egui::Visuals::light()),
        }
    }

    if !open {
        app.show_preferences_dialog = false;
    }
}

pub(super) fn render_status_bar(app: &mut ZephyrApp, ctx: &egui::Context) -> ZoomAction {
    let mut zoom_action = ZoomAction::None;

    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(8.0);

            ui.label("Status:");
            if !app.status_message.is_empty() {
                ui.label(&app.status_message);
            } else {
                ui.label("Ready");
            }

            if app.atlas.is_some() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);

                    let reset_response = ui.button("Reset");
                    if reset_response.clicked() {
                        zoom_action = ZoomAction::Reset;
                    }
                    let button_size = reset_response.rect.size();

                    if ui.add_sized(button_size, egui::Button::new(MAGNIFYING_GLASS_PLUS)).clicked() {
                        zoom_action = ZoomAction::ZoomIn;
                    }
                    if ui.add_sized(button_size, egui::Button::new(MAGNIFYING_GLASS_MINUS)).clicked() {
                        zoom_action = ZoomAction::ZoomOut;
                    }

                    let current_zoom = match app.current_tab {
                        AppTab::Preview => app.zoom_level,
                        AppTab::SpriteEditor => app.sprite_editor_zoom,
                        AppTab::AnimationEditor => app.anim_preview_zoom,
                    };

                    let mut zoom_slider = (current_zoom * 100.0) as i32;
                    if ui.add(egui::Slider::new(&mut zoom_slider, 10..=1000).suffix("%")).changed() {
                        let new_zoom = zoom_slider as f32 / 100.0;
                        match app.current_tab {
                            AppTab::Preview => app.zoom_level = new_zoom,
                            AppTab::SpriteEditor => app.sprite_editor_zoom = new_zoom,
                            AppTab::AnimationEditor => app.anim_preview_zoom = new_zoom,
                        }
                    }

                    ui.label("Zoom:");
                });
            }
        });
    });

    zoom_action
}

pub(super) fn render_central_panel(app: &mut ZephyrApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
        .show(ctx, |ui| {
            // Collect which rect the active tab occupies so we can draw the
            // separator with a gap under it, giving the "tab connected to
            // content" visual.
            let mut active_rect: Option<Rect> = None;

            ui.horizontal(|ui| {
                // Zero the inter-widget gap so tabs sit flush against each
                // other and against the panel's left edge with no gutter.
                ui.spacing_mut().item_spacing.x = 0.0;
                for (tab, label) in [
                    (AppTab::Preview, "Atlas Preview"),
                    (AppTab::SpriteEditor, "Sprite Editor"),
                    (AppTab::AnimationEditor, "Animations"),
                ] {
                    let selected = app.current_tab == tab;
                    let (rect, response) = render_tab(ui, label, selected);
                    if selected {
                        active_rect = Some(rect);
                    }
                    if response.clicked() {
                        app.current_tab = tab;
                        if tab != AppTab::AnimationEditor {
                            app.anim_playing = false;
                            app.anim_last_time = None;
                        }
                    }
                }
            });

            // Draw separator as two segments that leave a gap under the active
            // tab, so the tab background visually merges with the content area.
            let sep_y = ui.cursor().min.y + 0.5;
            let full_left = ui.min_rect().left();
            let full_right = ui.min_rect().right();
            let sep_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
            let sep_stroke = egui::Stroke::new(1.0, sep_color);

            match active_rect {
                Some(tab_rect) => {
                    // Left segment up to the active tab.
                    if tab_rect.left() > full_left {
                        ui.painter()
                            .line_segment([Pos2::new(full_left, sep_y), Pos2::new(tab_rect.left(), sep_y)], sep_stroke);
                    }
                    // Right segment from the active tab onward.
                    if tab_rect.right() < full_right {
                        ui.painter()
                            .line_segment([Pos2::new(tab_rect.right(), sep_y), Pos2::new(full_right, sep_y)], sep_stroke);
                    }
                }
                None => {
                    ui.painter().line_segment([Pos2::new(full_left, sep_y), Pos2::new(full_right, sep_y)], sep_stroke);
                }
            }

            // Reserve the separator's vertical space so content starts below it.
            ui.add_space(4.0);

            match app.current_tab {
                AppTab::Preview => super::preview_panel::render_content(app, ui),
                AppTab::SpriteEditor => super::sprite_panel::render_content(app, ui, ctx),
                AppTab::AnimationEditor => super::animation_panel::render_content(app, ui, ctx),
            }
        });
}

fn render_tab(ui: &mut Ui, label: &str, selected: bool) -> (Rect, egui::Response) {
    let h_pad = 14.0_f32;
    let v_pad = 7.0_f32;
    let tab_w = label.chars().count() as f32 * 7.5 + h_pad * 2.0;
    let tab_h = ui.text_style_height(&egui::TextStyle::Body) + v_pad * 2.0;
    let tab_size = vec2(tab_w, tab_h);

    let (rect, response) = ui.allocate_exact_size(tab_size, egui::Sense::click());

    if !ui.is_rect_visible(rect) {
        return (rect, response);
    }

    let top_rounded = egui::CornerRadius { nw: 4, ne: 4, sw: 0, se: 0 };

    let bg = if selected {
        ui.visuals().extreme_bg_color
    } else if response.hovered() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 10)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, top_rounded, bg);

    if selected {
        let border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let stroke = egui::Stroke::new(1.0, border_color);
        ui.painter().line_segment([rect.left_top(), rect.right_top()], stroke);
        ui.painter().line_segment([rect.left_top(), rect.left_bottom()], stroke);
        ui.painter().line_segment([rect.right_top(), rect.right_bottom()], stroke);
        // No bottom
        // The open bottom is what creates the visual connection between the
        // tab and the content panel.
    }

    let text_color = if selected {
        ui.visuals().strong_text_color()
    } else if response.hovered() {
        ui.visuals().text_color()
    } else {
        ui.visuals().weak_text_color()
    };
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, label, egui::FontId::proportional(13.0), text_color);

    (rect, response)
}
