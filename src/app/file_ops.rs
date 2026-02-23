// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use crate::loader::load_images;
use crate::types::{Directory, ImageNode, NodeId, TextureFormat, TreeNode, ZephyrProject};
use rfd::FileDialog;

use std::path::{Path, PathBuf};

use super::ZephyrApp;

pub(super) fn new_project(app: &mut ZephyrApp) {
    app.tree_root = TreeNode::Directory(Directory {
        id: 0,
        name: "Project".to_string(),
        children: Vec::new(),
    });
    app.next_id = 1;
    app.selected_nodes.clear();
    app.sprite_properties.clear();
    app.animations.clear();
    app.selected_animation_idx = None;
    app.anim_current_frame = 0;
    app.anim_frame_elapsed_ms = 0.0;
    app.anim_last_time = None;
    app.anim_playing = false;
    app.atlas = None;
    app.atlas_texture = None;
    app.atlas_size = None;
    app.atlas_efficiency = None;
    app.atlas_sprite_count = None;
    app.thumbnails.clear();
    app.project_path = None;
    app.has_unsaved_changes = false;
    app.status_message = "New project created".into();
}

pub(super) fn add_images(app: &mut ZephyrApp) {
    if let Some(paths) = FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
        .set_title("Select Images")
        .pick_files()
    {
        let new_images = load_images(&paths);
        let count = new_images.len();

        if let TreeNode::Directory(ref mut root) = app.tree_root {
            for img in new_images {
                let img_node = TreeNode::Image(ImageNode { id: app.next_id, image: img });
                app.next_id += 1;
                root.children.push(img_node);
            }
        }

        app.atlas = None;
        app.atlas_texture = None;
        app.has_unsaved_changes = true;
        app.status_message = format!("Added {} images", count);
        app.pack_atlas();
    }
}

pub(super) fn import_directory(app: &mut ZephyrApp) {
    if let Some(dir_path) = FileDialog::new().set_title("Select Directory to Import").pick_folder() {
        let (image_count, dir_count) = process_directory_recursive(app, &dir_path, 0);

        if image_count > 0 || dir_count > 0 {
            app.atlas = None;
            app.atlas_texture = None;
            app.has_unsaved_changes = true;

            app.status_message = if dir_count > 0 {
                format!("Imported {} directories with {} images", dir_count, image_count)
            } else {
                format!("Imported {} images", image_count)
            };

            app.pack_atlas();
        } else {
            app.status_message = "No images found in selected directory".into();
        }
    }
}

pub(super) fn save_project(app: &mut ZephyrApp) {
    if let Some(path) = app.project_path.clone() {
        save_project_to_path(app, &path, false);
    } else {
        if app.tree_root.collect_images().is_empty() {
            app.status_message = "No images to save in project".into();
            return;
        }
        if let Some(project_path) = FileDialog::new()
            .set_file_name("project.zephyr")
            .add_filter("Zephyr Project", &["zephyr"])
            .set_title("Save Project")
            .save_file()
        {
            save_project_to_path(app, &project_path, false);
        }
    }
}

pub(super) fn save_project_as(app: &mut ZephyrApp) {
    if app.tree_root.collect_images().is_empty() {
        app.status_message = "No images to save in project".into();
        return;
    }

    if let Some(project_path) = FileDialog::new()
        .set_file_name("project.zephyr")
        .add_filter("Zephyr Project", &["zephyr"])
        .set_title("Save Project As")
        .save_file()
    {
        save_project_to_path(app, &project_path, false);
    }
}

fn save_project_to_path(app: &mut ZephyrApp, project_path: &Path, silent: bool) {
    let mut project = ZephyrProject::new(app.settings.clone(), &app.tree_root, app.sprite_properties.clone(), app.animations.clone());

    if let Some(project_dir) = project_path.parent() {
        project.tree_root.relativize_paths(project_dir);
    }

    match serde_json::to_string_pretty(&project) {
        Ok(json) => {
            if let Err(e) = std::fs::write(project_path, json) {
                app.status_message = format!("Failed to save project: {}", e);
            } else {
                app.project_path = Some(project_path.to_path_buf());
                app.has_unsaved_changes = false;
                if !silent {
                    app.status_message = format!("Project saved to {}", project_path.display());
                }
            }
        }
        Err(e) => {
            app.status_message = format!("Failed to serialize project: {}", e);
        }
    }
}

pub(super) fn auto_save(app: &mut ZephyrApp) {
    if let Some(path) = app.project_path.clone() {
        save_project_to_path(app, &path, true);
    }
}

pub(super) fn load_project(app: &mut ZephyrApp) {
    if let Some(project_path) = FileDialog::new().add_filter("Zephyr Project", &["zephyr"]).set_title("Load Project").pick_file() {
        match std::fs::read_to_string(&project_path) {
            Ok(json) => {
                match serde_json::from_str::<ZephyrProject>(&json) {
                    Ok(mut project) => {
                        app.selected_nodes.clear();
                        app.atlas = None;
                        app.atlas_texture = None;
                        app.thumbnails.clear();

                        app.settings = project.settings;

                        app.settings.validate_formats();

                        app.sprite_properties = project.sprite_properties;
                        app.animations = project.animations;

                        app.anim_playing = false;
                        app.anim_current_frame = 0;
                        app.anim_frame_elapsed_ms = 0.0;
                        app.anim_last_time = None;
                        app.selected_animation_idx = if app.animations.is_empty() { None } else { Some(0) };

                        if let Some(project_dir) = project_path.parent() {
                            project.tree_root.absolutize_paths(project_dir);
                        }

                        let (loaded, failed) = project.tree_root.reload_images();

                        app.tree_root = project.tree_root;
                        // Advance past the highest ID already present to avoid collisions.
                        app.next_id = find_max_id(&app.tree_root) + 1;
                        app.project_path = Some(project_path.clone());
                        app.has_unsaved_changes = false;

                        if loaded > 0 {
                            let message = if failed > 0 {
                                format!("Loaded project with {} images ({} failed to load)", loaded, failed)
                            } else {
                                format!("Loaded project with {} images", loaded)
                            };
                            app.status_message = message;
                            app.pack_atlas();
                        } else {
                            app.status_message = "Project loaded but no images could be found".into();
                        }
                    }
                    Err(e) => {
                        app.status_message = format!("Failed to parse project file: {}", e);
                    }
                }
            }
            Err(e) => {
                app.status_message = format!("Failed to read project file: {}", e);
            }
        }
    }
}

fn find_max_id(node: &TreeNode) -> NodeId {
    match node {
        TreeNode::Directory(dir) => {
            let child_max = dir.children.iter().map(find_max_id).max().unwrap_or(0);
            dir.id.max(child_max)
        }
        TreeNode::Image(img) => img.id,
    }
}

pub(super) fn export_atlas(app: &mut ZephyrApp) {
    if let Some(atlas) = &app.atlas {
        let (extension, filter_name, extensions) = get_export_file_info(app.settings.texture_format);

        let default_filename = format!("atlas.{}", extension);

        if let Some(path) = FileDialog::new()
            .set_file_name(&default_filename)
            .add_filter(filter_name, extensions)
            .set_title("Export Atlas")
            .save_file()
        {
            let path_str = path.to_string_lossy();

            let base_path = path_str.trim_end_matches(&format!(".{}", extension)).trim_end_matches(".json");

            match crate::export::export_atlas(atlas, base_path, &app.settings, &app.sprite_properties, &app.animations) {
                Ok(_) => {
                    app.status_message = format!(
                        "Exported to {}.{} ({:?}, {:?})",
                        base_path, extension, app.settings.texture_format, app.settings.pixel_format
                    );
                }
                Err(e) => {
                    app.status_message = format!("Export failed: {}", e);
                }
            }
        }
    } else {
        app.status_message = "No atlas to export. Pack first!".into();
    }
}

fn get_export_file_info(format: TextureFormat) -> (&'static str, &'static str, &'static [&'static str]) {
    match format {
        TextureFormat::Png => ("png", "PNG Image", &["png"]),
        TextureFormat::Jpeg => ("jpg", "JPEG Image", &["jpg", "jpeg"]),
        TextureFormat::Bmp => ("bmp", "BMP Image", &["bmp"]),
        TextureFormat::Tga => ("tga", "TGA Image", &["tga"]),
        TextureFormat::Tiff => ("tiff", "TIFF Image", &["tiff", "tif"]),
        TextureFormat::WebP => ("webp", "WebP Image", &["webp"]),
    }
}

pub(super) fn clear(app: &mut ZephyrApp) {
    app.tree_root = TreeNode::Directory(Directory {
        id: 0,
        name: "Project".to_string(),
        children: Vec::new(),
    });
    app.next_id = 1;
    app.selected_nodes.clear();
    app.atlas = None;
    app.atlas_texture = None;
    app.thumbnails.clear();
    app.status_message = "Cleared".into();
}

pub(super) fn handle_dropped_paths(app: &mut ZephyrApp, paths: Vec<PathBuf>) {
    let target_id = app.drop_target_directory.unwrap_or(0);
    let mut total_images = 0;
    let mut total_dirs = 0;

    for path in paths {
        if path.is_dir() {
            let (img_count, dir_count) = process_directory_recursive(app, &path, target_id);
            total_images += img_count;
            total_dirs += dir_count;
        } else if path.is_file() {
            if let Ok(img) = crate::loader::load_image(&path) {
                let img_node = TreeNode::Image(ImageNode { id: app.next_id, image: img });
                app.next_id += 1;

                if app.tree_root.insert(target_id, img_node) {
                    total_images += 1;
                }
            }
        }
    }

    if total_images > 0 || total_dirs > 0 {
        app.atlas = None;
        app.atlas_texture = None;
        app.has_unsaved_changes = true;

        app.status_message = if total_dirs > 0 {
            format!("Added {} directories with {} images", total_dirs, total_images)
        } else {
            format!("Added {} images", total_images)
        };

        app.pack_atlas();
    }

    app.drop_target_directory = None;
}

fn process_directory_recursive(app: &mut ZephyrApp, dir_path: &Path, parent_id: NodeId) -> (usize, usize) {
    let mut image_count = 0;
    let mut dir_count = 0;

    let dir_name = dir_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .unwrap_or_else(|| "folder".to_string());

    let dir_id = app.next_id;
    app.next_id += 1;

    let new_dir = TreeNode::Directory(Directory {
        id: dir_id,
        name: dir_name,
        children: Vec::new(),
    });

    if !app.tree_root.insert(parent_id, new_dir) {
        eprintln!("Failed to insert directory into parent {}", parent_id);
        return (0, 0);
    }

    dir_count += 1;

    let entries = match std::fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read directory {:?}: {}", dir_path, e);
            return (image_count, dir_count);
        }
    };

    let mut image_paths = Vec::new();
    let mut subdirs = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            subdirs.push(path);
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if matches!(ext_str.as_str(), "png" | "jpg" | "jpeg" | "bmp") {
                    image_paths.push(path);
                }
            }
        }
    }

    let images = load_images(&image_paths);
    for img in images {
        let img_node = TreeNode::Image(ImageNode { id: app.next_id, image: img });
        app.next_id += 1;

        if app.tree_root.insert(dir_id, img_node) {
            image_count += 1;
        }
    }

    for subdir in subdirs {
        let (sub_imgs, sub_dirs) = process_directory_recursive(app, &subdir, dir_id);
        image_count += sub_imgs;
        dir_count += sub_dirs;
    }

    (image_count, dir_count)
}
