// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use crate::atlas::{apply_trim, create_atlas_texture};
use crate::geometry::{CircleHandle, RectHandle};
use crate::packing::pack_images;
use crate::types::{Animation, Directory, HitboxShape, NodeId, PackSettings, PackedAtlas, PlaybackDirection, SpriteProperties, TreeNode, TrimMode};
use egui::TextureHandle;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::thumbnails::ThumbnailCache;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum AppTab {
    Preview,
    SpriteEditor,
    AnimationEditor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThemePreference {
    Auto,
    Dark,
    Light,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundStyle {
    Checkerboard,
    SolidColor,
    Off,
}

pub(crate) struct AppPreferences {
    pub(crate) theme: ThemePreference,
    pub(crate) background_style: BackgroundStyle,
    pub(crate) solid_bg_color: egui::Color32,
    pub(crate) draw_border: bool,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            theme: ThemePreference::Auto,
            background_style: BackgroundStyle::Checkerboard,
            solid_bg_color: egui::Color32::from_rgb(64, 64, 64),
            draw_border: false,
        }
    }
}

#[derive(Clone)]
pub(crate) struct SpriteDragPayload(pub(crate) Vec<String>);

#[derive(Clone, Copy)]
pub(crate) struct FrameDragPayload(pub(crate) usize);

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum HitboxDragTarget {
    RectHandle(RectHandle),
    CircleHandle(CircleHandle),
    PolygonVertex(usize),
    Body,
}

pub(crate) struct HitboxDragState {
    pub(crate) target: HitboxDragTarget,
    pub(crate) original: HitboxShape,
    pub(crate) accumulated_dx: f32,
    pub(crate) accumulated_dy: f32,
}

pub(crate) struct ZephyrApp {
    // Tree
    pub(crate) tree_root: TreeNode,
    pub(crate) next_id: NodeId,
    pub(crate) selected_nodes: HashSet<NodeId>,

    // Pack settings and result
    pub(crate) settings: PackSettings,
    pub(crate) atlas: Option<PackedAtlas>,
    pub(crate) atlas_texture: Option<TextureHandle>,
    pub(crate) atlas_efficiency: Option<f32>,
    pub(crate) atlas_size: Option<(u32, u32)>,
    pub(crate) atlas_sprite_count: Option<usize>,
    pub(crate) status_message: String,

    // Per-sprite metadata (keyed by sprite name)
    pub(crate) sprite_properties: HashMap<String, SpriteProperties>,

    // Animations
    pub(crate) animations: Vec<Animation>,
    pub(crate) selected_animation_idx: Option<usize>,

    // Animation playback state
    pub(crate) anim_playing: bool,
    pub(crate) anim_current_frame: usize,
    pub(crate) anim_frame_elapsed_ms: f64,
    pub(crate) anim_last_time: Option<f64>,
    pub(crate) anim_ping_pong_forward: bool,

    // Sprite editor state
    pub(crate) polygon_in_progress: Vec<[f32; 2]>,
    pub(crate) is_drawing_polygon: bool,
    pub(crate) sprite_editor_zoom: f32,
    pub(crate) sprite_editor_pan: egui::Vec2,
    pub(crate) hitbox_drag: Option<HitboxDragState>,
    pub(crate) pivot_drag: bool,
    pub(crate) pivot_drag_start: [f32; 2],
    pub(crate) pivot_drag_delta: egui::Vec2,

    // Index of the polygon vertex targeted by the active right-click context menu.
    // `None` when no vertex context menu is open.
    pub(crate) polygon_ctx_menu_vertex: Option<usize>,

    // Animation preview state
    pub(crate) anim_preview_zoom: f32,
    pub(crate) anim_preview_pan: egui::Vec2,

    // Tab
    pub(crate) current_tab: AppTab,

    // UI state
    pub(crate) show_create_dir_modal: bool,
    pub(crate) new_dir_name: String,
    pub(crate) pending_parent_id: Option<NodeId>,
    pub(crate) show_clear_confirmation_modal: bool,
    pub(crate) renaming_node_id: Option<NodeId>,
    pub(crate) rename_buffer: String,
    pub(crate) renaming_animation_idx: Option<usize>,
    pub(crate) animation_rename_buffer: String,
    pub(crate) show_new_animation_modal: bool,
    pub(crate) new_animation_name: String,
    pub(crate) zoom_level: f32,
    pub(crate) pan_offset: egui::Vec2,
    pub(crate) drop_target_directory: Option<NodeId>,
    pub(crate) drag_select_start: Option<egui::Pos2>,
    pub(crate) drag_select_current: Option<egui::Pos2>,
    pub(crate) show_invalid_polygon_placement_modal: bool,
    pub(crate) show_about_dialog: bool,
    pub(crate) show_preferences_dialog: bool,
    pub(crate) prefs: AppPreferences,
    pub(crate) project_path: Option<PathBuf>,
    pub(crate) has_unsaved_changes: bool,

    // Caches
    pub(crate) thumbnails: ThumbnailCache,

    // Mascot texture for the About dialog (loaded once at startup)
    pub(super) mascot_texture: Option<TextureHandle>,
}

impl Default for ZephyrApp {
    fn default() -> Self {
        Self {
            tree_root: TreeNode::Directory(Directory {
                id: 0,
                name: "Project".to_string(),
                children: Vec::new(),
            }),
            next_id: 1,
            selected_nodes: HashSet::new(),
            settings: PackSettings::default(),
            atlas: None,
            atlas_texture: None,
            atlas_efficiency: None,
            atlas_size: None,
            atlas_sprite_count: None,
            status_message: String::new(),
            sprite_properties: HashMap::new(),
            animations: Vec::new(),
            selected_animation_idx: None,
            anim_playing: false,
            anim_current_frame: 0,
            anim_frame_elapsed_ms: 0.0,
            anim_last_time: None,
            anim_ping_pong_forward: true,
            polygon_in_progress: Vec::new(),
            is_drawing_polygon: false,
            sprite_editor_zoom: 1.0,
            sprite_editor_pan: egui::Vec2::ZERO,
            hitbox_drag: None,
            pivot_drag: false,
            pivot_drag_start: [0.0, 0.0],
            pivot_drag_delta: egui::Vec2::ZERO,
            polygon_ctx_menu_vertex: None,
            anim_preview_zoom: 1.0,
            anim_preview_pan: egui::Vec2::ZERO,
            current_tab: AppTab::Preview,
            show_create_dir_modal: false,
            new_dir_name: String::new(),
            pending_parent_id: None,
            show_clear_confirmation_modal: false,
            renaming_node_id: None,
            rename_buffer: String::new(),
            renaming_animation_idx: None,
            animation_rename_buffer: String::new(),
            show_new_animation_modal: false,
            new_animation_name: String::new(),
            zoom_level: 1.0,
            pan_offset: egui::Vec2::ZERO,
            drop_target_directory: None,
            drag_select_start: None,
            drag_select_current: None,
            show_invalid_polygon_placement_modal: false,
            show_about_dialog: false,
            show_preferences_dialog: false,
            prefs: AppPreferences::default(),
            project_path: None,
            has_unsaved_changes: false,
            thumbnails: ThumbnailCache::new(),
            mascot_texture: None,
        }
    }
}

impl ZephyrApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        let mut app = Self::default();

        const MASCOT_BYTES: &[u8] = include_bytes!("../assets/zephyr_mascot.png");
        if let Ok(img) = image::load_from_memory(MASCOT_BYTES) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
            app.mascot_texture = Some(cc.egui_ctx.load_texture("zephyr_mascot", color_image, egui::TextureOptions::LINEAR));
        }

        app
    }

    pub(crate) fn selected_sprite_name(&self) -> Option<String> {
        if self.selected_nodes.len() != 1 {
            return None;
        }
        let &id = self.selected_nodes.iter().next()?;
        self.tree_root.iter_images().find(|img| img.id == id).map(|img| img.image.name.clone())
    }

    pub(crate) fn pack_atlas(&mut self) {
        let all_images = self.tree_root.collect_images();

        if all_images.is_empty() {
            self.status_message = "No images to pack".into();
            self.atlas_efficiency = None;
            self.atlas_size = None;
            self.atlas_sprite_count = None;
            return;
        }

        let image_count = all_images.len();
        let images_to_pack = if self.settings.trim_mode == TrimMode::Off {
            all_images
        } else {
            apply_trim(all_images)
        };

        match pack_images(&images_to_pack, &self.settings) {
            Some(mut atlas) => {
                atlas.texture = create_atlas_texture(
                    &atlas.placements,
                    &images_to_pack,
                    atlas.width,
                    atlas.height,
                    self.settings.extrude,
                    self.settings.alpha_processing,
                );

                let sprite_count = atlas.placements.len();
                let total_sprite_area: u32 = atlas.placements.iter().map(|p| p.width * p.height).sum();
                let atlas_area = atlas.width * atlas.height;
                let efficiency = if atlas_area > 0 {
                    (total_sprite_area as f32 / atlas_area as f32) * 100.0
                } else {
                    0.0
                };

                self.atlas_efficiency = Some(efficiency);
                self.atlas_size = Some((atlas.width, atlas.height));
                self.atlas_sprite_count = Some(sprite_count);
                self.status_message = format!("Packed {} images (Efficiency {:.1}%)", image_count, efficiency);
                self.atlas = Some(atlas);
                self.atlas_texture = None;
            }
            None => {
                self.atlas_efficiency = None;
                self.atlas_size = None;
                self.atlas_sprite_count = None;
                self.status_message = "Images don't fit in 4096x4096 atlas! Some images are too large.".into();
            }
        }
    }

    pub(super) fn ensure_atlas_texture(&mut self, ctx: &egui::Context) {
        if self.atlas_texture.is_some() {
            return;
        }
        let Some(atlas) = &self.atlas else {
            return;
        };
        let size = [atlas.width as usize, atlas.height as usize];
        let pixels = atlas.texture.as_flat_samples();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        let texture_options = egui::TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Linear,
            ..Default::default()
        };
        self.atlas_texture = Some(ctx.load_texture("atlas_preview", color_image, texture_options));
    }

    pub(crate) fn stop_playback(&mut self) {
        self.anim_playing = false;
        self.anim_last_time = None;
        self.anim_frame_elapsed_ms = 0.0;
    }

    pub(super) fn tick_anim_playback(&mut self, ctx: &egui::Context) {
        if !self.anim_playing {
            self.anim_last_time = None;
            return;
        }

        let Some(anim_idx) = self.selected_animation_idx else {
            self.anim_playing = false;
            return;
        };

        // Bail if anim_idx is stale after a deletion or the animation has no frames.
        let frames_empty = self.animations.get(anim_idx).is_none_or(|a| a.frames.is_empty());
        if frames_empty {
            self.anim_playing = false;
            return;
        }

        let now = ctx.input(|i| i.time);
        let delta_ms = match self.anim_last_time {
            Some(prev) => ((now - prev) * 1000.0).max(0.0),
            None => 0.0,
        };
        self.anim_last_time = Some(now);
        self.anim_frame_elapsed_ms += delta_ms;

        // Scope the immutable borrow so advance_anim_frame can take &mut self below.
        let current_delay = {
            let Some(anim) = self.animations.get(anim_idx) else {
                return;
            };
            let current_frame = self.anim_current_frame.min(anim.frames.len().saturating_sub(1));
            let Some(frame) = anim.frames.get(current_frame) else {
                return;
            };
            frame.delay_ms as f64
        };

        if self.anim_frame_elapsed_ms >= current_delay {
            self.anim_frame_elapsed_ms -= current_delay;
            self.advance_anim_frame(anim_idx);
        }

        ctx.request_repaint();
    }

    fn advance_anim_frame(&mut self, anim_idx: usize) {
        let (frame_count, direction, loop_enabled) = match self.animations.get(anim_idx) {
            Some(anim) if !anim.frames.is_empty() => (anim.frames.len(), anim.direction, anim.loop_enabled),
            _ => return,
        };

        match direction {
            PlaybackDirection::Forward => {
                if self.anim_current_frame + 1 < frame_count {
                    self.anim_current_frame += 1;
                } else if loop_enabled {
                    self.anim_current_frame = 0;
                } else {
                    self.anim_playing = false;
                }
            }
            PlaybackDirection::Reverse => {
                if self.anim_current_frame > 0 {
                    self.anim_current_frame -= 1;
                } else if loop_enabled {
                    self.anim_current_frame = frame_count - 1;
                } else {
                    self.anim_playing = false;
                }
            }
            PlaybackDirection::PingPong => {
                if self.anim_ping_pong_forward {
                    if self.anim_current_frame + 1 < frame_count {
                        self.anim_current_frame += 1;
                    } else {
                        self.anim_ping_pong_forward = false;
                        if frame_count > 1 {
                            self.anim_current_frame = frame_count - 2;
                        }
                    }
                } else if self.anim_current_frame > 0 {
                    self.anim_current_frame -= 1;
                } else {
                    if loop_enabled {
                        self.anim_ping_pong_forward = true;
                        if frame_count > 1 {
                            self.anim_current_frame = 1;
                        }
                    } else {
                        self.anim_playing = false;
                    }
                }
            }
        }
    }

    pub(crate) fn remove_selected(&mut self) {
        if self.selected_nodes.contains(&0) {
            self.show_clear_confirmation_modal = true;
            return;
        }

        let ids: Vec<NodeId> = self.selected_nodes.iter().copied().collect();
        let count = self.remove_nodes_with_cleanup(&ids);
        self.selected_nodes.clear();

        if count > 0 {
            self.has_unsaved_changes = true;
            self.status_message = format!("Removed {} item(s)", count);
            self.pack_atlas();
        }
    }

    pub(crate) fn remove_node(&mut self, id: NodeId) {
        let count = self.remove_nodes_with_cleanup(&[id]);
        self.selected_nodes.remove(&id);

        if count > 0 {
            self.has_unsaved_changes = true;
            self.status_message = "Deleted".into();
            self.pack_atlas();
        }
    }

    fn remove_nodes_with_cleanup(&mut self, ids: &[NodeId]) -> usize {
        // Snapshot image names before removal so we can detect what disappears.
        let names_before: Vec<String> = self.tree_root.iter_images().map(|img| img.image.name.clone()).collect();

        let mut removed = 0;
        for &id in ids {
            if self.tree_root.remove(id).is_some() {
                self.thumbnails.remove(id);
                removed += 1;
            }
        }

        if removed > 0 {
            let names_after: HashSet<String> = self.tree_root.iter_images().map(|img| img.image.name.clone()).collect();

            // Clean up metadata for every image name that is no longer present.
            for name in &names_before {
                if !names_after.contains(name) {
                    self.sprite_properties.remove(name);
                    for anim in &mut self.animations {
                        anim.frames.retain(|f| &f.sprite_name != name);
                    }
                }
            }

            // Clamp anim_current_frame to the new frame count.
            if let Some(anim_idx) = self.selected_animation_idx {
                if let Some(anim) = self.animations.get(anim_idx) {
                    let last = anim.frames.len().saturating_sub(1);
                    if self.anim_current_frame > last {
                        self.anim_current_frame = last;
                    }
                }
            }

            self.atlas = None;
            self.atlas_texture = None;
        }

        removed
    }

    pub(crate) fn start_rename(&mut self, id: NodeId) {
        if let Some(name) = self.tree_root.node_name(id) {
            self.rename_buffer = name.to_string();
            self.renaming_node_id = Some(id);
        }
    }

    pub(crate) fn finalize_rename(&mut self) {
        let Some(id) = self.renaming_node_id else {
            return;
        };
        let new_name = self.rename_buffer.trim().to_string();

        // empty name is not accepted.
        if new_name.is_empty() {
            self.renaming_node_id = None;
            return;
        }

        let is_image = self.tree_root.iter_images().any(|img| img.id == id);
        let old_name = self.tree_root.node_name(id).map(|s| s.to_string());

        if self.tree_root.rename_node(id, new_name.clone()) {
            if is_image {
                if let Some(old) = old_name {
                    if old != new_name {
                        // Migrate sprite_properties to the new key.
                        if let Some(props) = self.sprite_properties.remove(&old) {
                            self.sprite_properties.insert(new_name.clone(), props);
                        }
                        // Update every animation frame that referenced the old name.
                        for anim in &mut self.animations {
                            for frame in &mut anim.frames {
                                if frame.sprite_name == old {
                                    frame.sprite_name = new_name.clone();
                                }
                            }
                        }
                    }
                }
            }
            self.has_unsaved_changes = true;
            self.status_message = format!("Renamed to '{}'", new_name);
        }

        self.renaming_node_id = None;
    }

    pub(crate) fn open_new_animation_modal(&mut self) {
        self.new_animation_name = String::new();
        self.show_new_animation_modal = true;
    }

    pub(crate) fn finalize_new_animation(&mut self) -> Option<usize> {
        let name = self.new_animation_name.trim().to_string();
        if name.is_empty() {
            return None;
        }
        self.animations.push(Animation::new(name));
        let new_idx = self.animations.len() - 1;
        self.selected_animation_idx = Some(new_idx);
        self.stop_playback();
        self.show_new_animation_modal = false;
        self.has_unsaved_changes = true;
        Some(new_idx)
    }

    pub(crate) fn start_rename_animation(&mut self, idx: usize) {
        if let Some(anim) = self.animations.get(idx) {
            self.animation_rename_buffer = anim.name.clone();
            self.renaming_animation_idx = Some(idx);
        }
    }

    pub(crate) fn finalize_rename_animation(&mut self) {
        let Some(idx) = self.renaming_animation_idx else {
            return;
        };
        let new_name = self.animation_rename_buffer.trim().to_string();
        if new_name.is_empty() {
            self.renaming_animation_idx = None;
            return;
        }
        if let Some(anim) = self.animations.get_mut(idx) {
            anim.name = new_name.clone();
            self.has_unsaved_changes = true;
            self.status_message = format!("Renamed animation to '{}'", new_name);
        }
        self.renaming_animation_idx = None;
    }

    pub(crate) fn show_create_directory_at(&mut self, parent_id: NodeId) {
        self.pending_parent_id = Some(parent_id);
        self.new_dir_name = String::new();
        self.show_create_dir_modal = true;
    }

    pub(crate) fn finalize_create_directory(&mut self) {
        if !self.new_dir_name.trim().is_empty()
            && let Some(parent_id) = self.pending_parent_id
        {
            let new_dir = TreeNode::Directory(Directory {
                id: self.next_id,
                name: self.new_dir_name.trim().to_string(),
                children: Vec::new(),
            });
            self.next_id += 1;

            if self.tree_root.insert(parent_id, new_dir) {
                self.has_unsaved_changes = true;
                self.status_message = format!("Created directory '{}'", self.new_dir_name.trim());
            }
        }
        self.show_create_dir_modal = false;
        self.pending_parent_id = None;
    }

    pub(super) fn handle_input(&mut self, ctx: &egui::Context) {
        let (dropped_paths, save, load, export, escape, delete) = ctx.input(|i| {
            let paths: Vec<_> = i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect();
            let cmd = i.modifiers.command;
            (
                paths,
                cmd && i.key_pressed(egui::Key::S),
                cmd && i.key_pressed(egui::Key::O),
                cmd && i.key_pressed(egui::Key::E),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Delete),
            )
        });

        if !dropped_paths.is_empty() {
            super::file_ops::handle_dropped_paths(self, dropped_paths);
        }

        if save {
            super::file_ops::save_project(self);
        }
        if load {
            super::file_ops::load_project(self);
        }
        if export {
            super::file_ops::export_atlas(self);
        }

        if escape {
            if self.renaming_node_id.is_some() {
                // Escape reverts an in-progress tree-node rename.
                self.renaming_node_id = None;
            } else if self.renaming_animation_idx.is_some() {
                // Escape reverts an in-progress animation rename.
                self.renaming_animation_idx = None;
            } else if self.show_new_animation_modal {
                // Escape dismisses the new-animation naming modal.
                self.show_new_animation_modal = false;
            } else if !self.selected_nodes.is_empty() {
                self.selected_nodes.clear();
                // Also clear the TreeView's selection state
                let tree_id = egui::Id::new("source_tree");
                ctx.data_mut(|data| {
                    let state = data.get_persisted_mut_or_default::<egui_ltreeview::TreeViewState<NodeId>>(tree_id);
                    state.set_selected(Vec::<NodeId>::new());
                });
            }
        }

        // Delete removes selected nodes from anywhere in the app (atlas preview,
        // tree panel, animation editor, etc.) provided no text widget has keyboard
        // focus and no confirmation modal is waiting for a response.
        if delete && !ctx.wants_keyboard_input() && !self.selected_nodes.is_empty() && !self.show_clear_confirmation_modal {
            self.remove_selected();
        }
    }
}
