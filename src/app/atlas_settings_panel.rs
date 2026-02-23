// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use super::ZephyrApp;
use super::settings_panel::{render_collapsing_section, two_column_row};
use crate::types::{AlphaProcessing, PackingAlgorithm, ShelfSortBy, SortOrder, TextureFormat, TrimMode};

pub(super) fn render(app: &mut ZephyrApp, ui: &mut egui::Ui, margin: egui::Margin) {
    render_collapsing_section(ui, "Texture", margin, |ui| {
        render_texture_format(app, ui);
        ui.add_space(4.0);
        render_pixel_format(app, ui);
        ui.add_space(4.0);
        if app.settings.texture_format.has_compression_setting() {
            render_png_compression(app, ui);
            ui.add_space(4.0);
        }
        if app.settings.texture_format.has_quality_setting() {
            render_jpeg_quality(app, ui);
            ui.add_space(4.0);
        }
        render_alpha_processing(app, ui);
    });

    render_collapsing_section(ui, "Layout", margin, |ui| {
        render_algorithm_selector(app, ui);
        if app.settings.algorithm == PackingAlgorithm::Shelf {
            ui.add_space(4.0);
            render_shelf_sort_by(app, ui);
            ui.add_space(4.0);
            render_shelf_sort_order(app, ui);
        }
        ui.add_space(4.0);
        render_force_square_checkbox(app, ui);
        ui.add_space(4.0);
        render_grid_alignment_slider(app, ui);
    });

    render_collapsing_section(ui, "Sprites", margin, |ui| {
        render_trim_mode(app, ui);
        ui.add_space(4.0);
        render_extrude_slider(app, ui);
        ui.add_space(4.0);
        render_border_padding_slider(app, ui);
        ui.add_space(4.0);
        render_shape_padding_slider(app, ui);
    });
}

fn render_texture_format(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Texture Format:", |ui| {
        let previous_format = app.settings.texture_format;

        egui::ComboBox::from_id_salt("texture_format")
            .width(ui.available_width())
            .selected_text(app.settings.texture_format.name())
            .show_ui(ui, |ui| {
                let mut add_format = |ui: &mut egui::Ui, format: TextureFormat| {
                    let response = ui.selectable_value(&mut app.settings.texture_format, format, format.name());
                    if let Some(desc) = format.description() {
                        response.on_hover_text(desc);
                    }
                };
                add_format(ui, TextureFormat::Png);
                add_format(ui, TextureFormat::Jpeg);
                add_format(ui, TextureFormat::Bmp);
                add_format(ui, TextureFormat::Tga);
                add_format(ui, TextureFormat::Tiff);
                add_format(ui, TextureFormat::WebP);
            });

        if app.settings.texture_format != previous_format {
            app.settings.validate_formats();
        }
    });
}

fn render_pixel_format(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Pixel Format:", |ui| {
        let valid_formats = app.settings.texture_format.valid_pixel_formats();

        egui::ComboBox::from_id_salt("pixel_format")
            .width(ui.available_width())
            .selected_text(app.settings.pixel_format.name())
            .show_ui(ui, |ui| {
                for &format in valid_formats {
                    ui.selectable_value(&mut app.settings.pixel_format, format, format.name())
                        .on_hover_text(format.description());
                }
            });
    });
}

fn render_png_compression(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "PNG Compression:", |ui| {
        ui.add_sized(
            [ui.available_width(), ui.spacing().interact_size.y],
            egui::Slider::new(&mut app.settings.png_compression_level, 0..=9).show_value(true),
        );
    })
    .on_hover_text("Higher = smaller file, slower (0-9)");
}

fn render_jpeg_quality(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "JPEG Quality:", |ui| {
        ui.add_sized(
            [ui.available_width(), ui.spacing().interact_size.y],
            egui::Slider::new(&mut app.settings.jpeg_quality, 1..=100).show_value(true),
        );
    })
    .on_hover_text("Higher = better quality, larger file (1-100)");
}

fn render_alpha_processing(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Alpha Processing:", |ui| {
        egui::ComboBox::from_id_salt("alpha_processing")
            .width(ui.available_width())
            .selected_text(app.settings.alpha_processing.name())
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(&mut app.settings.alpha_processing, AlphaProcessing::None, AlphaProcessing::None.name())
                    .on_hover_text(AlphaProcessing::None.description())
                    .clicked()
                {
                    repack_if_needed(app);
                }
                if ui
                    .selectable_value(
                        &mut app.settings.alpha_processing,
                        AlphaProcessing::OptimizeCompression,
                        AlphaProcessing::OptimizeCompression.name(),
                    )
                    .on_hover_text(AlphaProcessing::OptimizeCompression.description())
                    .clicked()
                {
                    repack_if_needed(app);
                }
                if ui
                    .selectable_value(
                        &mut app.settings.alpha_processing,
                        AlphaProcessing::Premultiplied,
                        AlphaProcessing::Premultiplied.name(),
                    )
                    .on_hover_text(AlphaProcessing::Premultiplied.description())
                    .clicked()
                {
                    repack_if_needed(app);
                }
            });
    });
}

fn render_algorithm_selector(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Packing Algorithm:", |ui| {
        egui::ComboBox::from_id_salt("packing_algorithm")
            .width(ui.available_width())
            .selected_text(app.settings.algorithm.name())
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(&mut app.settings.algorithm, PackingAlgorithm::Shelf, PackingAlgorithm::Shelf.name())
                    .on_hover_text(PackingAlgorithm::Shelf.description())
                    .changed()
                {
                    repack_if_needed(app);
                }
                if ui
                    .selectable_value(&mut app.settings.algorithm, PackingAlgorithm::MaxRects, PackingAlgorithm::MaxRects.name())
                    .on_hover_text(PackingAlgorithm::MaxRects.description())
                    .changed()
                {
                    repack_if_needed(app);
                }
            });
    });
}

fn render_force_square_checkbox(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Force Square:", |ui| {
        if ui.checkbox(&mut app.settings.force_square, "").changed() {
            repack_if_needed(app);
        }
    })
    .on_hover_text("Force atlas to be square (width = height). Uncheck to allow rectangular atlases.");
}

fn render_trim_mode(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Trim Mode:", |ui| {
        egui::ComboBox::from_id_salt("trim_mode")
            .width(ui.available_width())
            .selected_text(app.settings.trim_mode.name())
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(&mut app.settings.trim_mode, TrimMode::Off, TrimMode::Off.name())
                    .on_hover_text(TrimMode::Off.description())
                    .clicked()
                {
                    repack_if_needed(app);
                }
                if ui
                    .selectable_value(&mut app.settings.trim_mode, TrimMode::Trim, TrimMode::Trim.name())
                    .on_hover_text(TrimMode::Trim.description())
                    .clicked()
                {
                    repack_if_needed(app);
                }
            });
    });
}

fn render_extrude_slider(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Extrude:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::Slider::new(&mut app.settings.extrude, 0..=8).show_value(true),
            )
            .changed()
        {
            repack_if_needed(app);
        }
    })
    .on_hover_text("Repeat edge pixels (reduces artifacts)");
}

fn render_border_padding_slider(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Border Padding:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::Slider::new(&mut app.settings.border_padding, 0..=16).show_value(true),
            )
            .changed()
        {
            repack_if_needed(app);
        }
    })
    .on_hover_text("Space around atlas edges");
}

fn render_shape_padding_slider(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Shape Padding:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::Slider::new(&mut app.settings.shape_padding, 0..=16).show_value(true),
            )
            .changed()
        {
            repack_if_needed(app);
        }
    })
    .on_hover_text("Space between sprites");
}

fn render_grid_alignment_slider(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Grid Alignment:", |ui| {
        if ui
            .add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                egui::Slider::new(&mut app.settings.grid_alignment, 0..=64).show_value(true),
            )
            .changed()
        {
            repack_if_needed(app);
        }
    })
    .on_hover_text("Snap each sprite's top-left corner to a grid. 0 = disabled. Common values: 2, 4, 8, 16, 32.");
}

fn render_shelf_sort_by(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Sort By:", |ui| {
        egui::ComboBox::from_id_salt("shelf_sort_by")
            .width(ui.available_width())
            .selected_text(app.settings.shelf_sort_by.name())
            .show_ui(ui, |ui| {
                let options = [
                    ShelfSortBy::Best,
                    ShelfSortBy::Directory,
                    ShelfSortBy::Name,
                    ShelfSortBy::Width,
                    ShelfSortBy::Height,
                    ShelfSortBy::Circumference,
                ];
                for option in options {
                    if ui
                        .selectable_value(&mut app.settings.shelf_sort_by, option, option.name())
                        .on_hover_text(option.description())
                        .clicked()
                    {
                        repack_if_needed(app);
                    }
                }
            });
    });
}

fn render_shelf_sort_order(app: &mut ZephyrApp, ui: &mut egui::Ui) {
    two_column_row(ui, "Sort Order:", |ui| {
        egui::ComboBox::from_id_salt("shelf_sort_order")
            .width(ui.available_width())
            .selected_text(app.settings.shelf_sort_order.name())
            .show_ui(ui, |ui| {
                for option in [SortOrder::Ascending, SortOrder::Descending] {
                    if ui.selectable_value(&mut app.settings.shelf_sort_order, option, option.name()).clicked() {
                        repack_if_needed(app);
                    }
                }
            });
    });
}

fn repack_if_needed(app: &mut ZephyrApp) {
    if !app.tree_root.collect_images().is_empty() {
        app.has_unsaved_changes = true;
        app.pack_atlas();
    }
}
