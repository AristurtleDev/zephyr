// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod atlas;
mod errors;
mod export;
mod geometry;
mod loader;
mod packing;
mod types;

use app::ZephyrApp;

fn main() -> Result<(), eframe::Error> {
    let icon = load_icon().map(std::sync::Arc::new);

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1024.0, 768.0])
        .with_title("Zephyr Texture Packer");

    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions { viewport, ..Default::default() };

    eframe::run_native("Zephyr", options, Box::new(|cc| Ok(Box::new(ZephyrApp::new(cc)))))
}

fn load_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("assets/icon.png");
    let image = image::load_from_memory(bytes).ok()?.into_rgba8();
    let width = image.width();
    let height = image.height();
    Some(egui::IconData { rgba: image.into_raw(), width, height })
}
