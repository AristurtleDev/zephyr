// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use crate::types::{NodeId, TreeNode};
use egui::Ui;
use egui_ltreeview::{NodeBuilder, TreeView, TreeViewBuilder};
use egui_phosphor::regular::{FOLDER, FOLDER_OPEN, PENCIL_SIMPLE, TRASH};

use super::ZephyrApp;
use super::thumbnails::ThumbnailCache;

enum ContextMenuAction {
    NewDirectory(NodeId),
    DeleteNode(NodeId),
    RenameNode(NodeId),
}

pub(super) fn render(app: &mut ZephyrApp, ctx: &egui::Context) {
    egui::SidePanel::left("images_panel")
        .default_width(250.0)
        .resizable(true)
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(0.0))
        .show(ctx, |ui| {
            let margin = ui.spacing().window_margin;

            ui.add_space(margin.top as f32);
            ui.horizontal(|ui| {
                ui.add_space(margin.left as f32);
                ui.heading("Source Images");
            });

            ui.separator();

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                render_tree_view(app, ui, ctx);
            });
        });
}

fn render_tree_view(app: &mut ZephyrApp, ui: &mut Ui, ctx: &egui::Context) {
    let mut context_menu_actions = Vec::new();
    let mut fallback_delete_ids: Vec<NodeId> = Vec::new();

    let tree_id = egui::Id::new("source_tree");

    let tree_root = &app.tree_root;
    let thumbnails = &mut app.thumbnails;

    let (tree_response, actions) = TreeView::new(tree_id)
        .allow_multi_selection(true)
        .allow_drag_and_drop(true)
        .fallback_context_menu(|ui, selected_nodes| {
            ui.set_min_width(180.0);
            ui.label(format!("{} items selected", selected_nodes.len()));
            ui.separator();
            if ui.button(format!("{} Delete Selected", TRASH)).clicked() {
                fallback_delete_ids.extend_from_slice(selected_nodes);
                ui.close();
            }
        })
        .show(ui, |builder| {
            render_node(builder, tree_root, &mut context_menu_actions, thumbnails, ctx);
        });

    for action in actions {
        match action {
            egui_ltreeview::Action::SetSelected(selection) => {
                app.selected_nodes = selection.into_iter().collect();
            }
            egui_ltreeview::Action::Move(dnd) => {
                handle_drag_drop(app, dnd);
            }
            egui_ltreeview::Action::Activate(activate) => {
                // Double-click or Enter: if exactly one image node is activated, open
                // it in the Sprite Editor tab and ensure it is selected.
                // For directory nodes, double-click triggers a rename.
                if let Some(&id) = activate.selected.first() {
                    let is_image = app.tree_root.iter_images().any(|img| img.id == id);
                    if is_image {
                        app.selected_nodes.clear();
                        app.selected_nodes.insert(id);
                        app.current_tab = super::AppTab::SpriteEditor;
                    } else {
                        // Directory double-click
                        if id != 0 {
                            app.start_rename(id);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // When the pointer is dragging from inside this panel, read which nodes the
    // TreeView itself considers to be dragged (via the now public get_dragged())
    // and use those as the sprite drag payload for the animation panel.
    let panel_rect = ui.clip_rect();
    if ctx.input(|i| i.pointer.is_decidedly_dragging()) {
        let drag_started_here = ctx.input(|i| i.pointer.press_origin().map(|o| panel_rect.contains(o)).unwrap_or(false));

        if drag_started_here {
            let dragged_nodes = ui.ctx().data_mut(|data| {
                data.get_persisted::<egui_ltreeview::TreeViewState<NodeId>>(tree_id)
                    .and_then(|s| s.get_dragged().cloned())
                    .unwrap_or_default()
            });
            if let Some(names) = node_ids_to_sprite_names(&app.tree_root, &dragged_nodes) {
                egui::DragAndDrop::set_payload(ctx, super::SpriteDragPayload(names));
            }
        }
    }

    for action in context_menu_actions {
        match action {
            ContextMenuAction::NewDirectory(parent_id) => {
                app.show_create_directory_at(parent_id);
            }
            ContextMenuAction::DeleteNode(node_id) => {
                app.remove_node(node_id);
            }
            ContextMenuAction::RenameNode(node_id) => {
                app.start_rename(node_id);
            }
        }
    }

    if !fallback_delete_ids.is_empty() {
        for id in &fallback_delete_ids {
            app.selected_nodes.insert(*id);
        }
        app.remove_selected();
    }

    // Delete key: remove selected nodes.
    // Only process Delete when the tree panel is focused/hovered and no modal is open.
    let tree_is_focused = tree_response.hovered();
    let no_modal_open = !app.show_create_dir_modal && app.renaming_node_id.is_none();

    if tree_is_focused && no_modal_open && ui.input(|i| i.key_pressed(egui::Key::Delete)) && !app.selected_nodes.is_empty() {
        app.remove_selected();
    }

    // F2 key: begin renaming the single selected node (skip root, id 0).
    // Only process F2 when the tree panel is focused/hovered and no modal is open.
    if tree_is_focused && no_modal_open && ui.input(|i| i.key_pressed(egui::Key::F2)) && app.selected_nodes.len() == 1 {
        if let Some(&id) = app.selected_nodes.iter().next() {
            if id != 0 {
                app.start_rename(id);
            }
        }
    }
}

fn render_node(builder: &mut TreeViewBuilder<NodeId>, node: &TreeNode, actions: &mut Vec<ContextMenuAction>, thumbnails: &mut ThumbnailCache, ctx: &egui::Context) {
    match node {
        TreeNode::Directory(dir) => {
            let node_builder = NodeBuilder::dir(dir.id)
                .label(&dir.name)
                .closer(|ui, state| {
                    let icon = if state.is_open { FOLDER_OPEN } else { FOLDER };
                    let color = if state.is_hovered {
                        ui.visuals().widgets.hovered.fg_stroke.color
                    } else {
                        ui.visuals().widgets.noninteractive.fg_stroke.color
                    };
                    ui.label(egui::RichText::new(icon).color(color));
                })
                .context_menu(|ui| {
                    ui.set_min_width(180.0);
                    ui.label(&dir.name);
                    ui.separator();

                    if ui.button(format!("{} New Directory", FOLDER)).clicked() {
                        actions.push(ContextMenuAction::NewDirectory(dir.id));
                        ui.close();
                    }

                    if dir.id != 0 && ui.button(format!("{} Rename", PENCIL_SIMPLE)).clicked() {
                        actions.push(ContextMenuAction::RenameNode(dir.id));
                        ui.close();
                    }

                    if dir.id != 0 && ui.button(format!("{} Delete Directory", TRASH)).clicked() {
                        actions.push(ContextMenuAction::DeleteNode(dir.id));
                        ui.close();
                    }
                });

            builder.node(node_builder);
            for child in &dir.children {
                render_node(builder, child, actions, thumbnails, ctx);
            }
            builder.close_dir();
        }
        TreeNode::Image(img_node) => {
            let label = format!("{} ({}x{})", img_node.image.name, img_node.image.width, img_node.image.height);

            let img_id = img_node.id;
            let thumbnail = thumbnails.get_or_insert(img_node.id, &img_node.image.data, ctx).clone();

            let node_builder = NodeBuilder::leaf(img_id)
                .label(&label)
                .icon(move |ui| {
                    let icon_w = ui.spacing().icon_width;
                    ui.add(egui::Image::new(&thumbnail).max_size(egui::vec2(icon_w, icon_w)));
                })
                .context_menu(|ui| {
                    ui.set_min_width(180.0);
                    ui.label(&img_node.image.name);
                    ui.separator();
                    ui.label(format!("Size: {}x{}", img_node.image.width, img_node.image.height));
                    ui.separator();

                    if ui.button(format!("{} Rename", PENCIL_SIMPLE)).clicked() {
                        actions.push(ContextMenuAction::RenameNode(img_node.id));
                        ui.close();
                    }

                    if ui.button(format!("{} Delete Image", TRASH)).clicked() {
                        actions.push(ContextMenuAction::DeleteNode(img_node.id));
                        ui.close();
                    }
                });

            builder.node(node_builder);
        }
    }
}

fn handle_drag_drop(app: &mut ZephyrApp, dnd: egui_ltreeview::DragAndDrop<NodeId>) {
    let mut moved_any = false;

    for source_node_id in &dnd.source {
        // Don't allow moving root
        if *source_node_id == 0 {
            continue;
        }

        // Remove from current location
        if let Some(node) = app.tree_root.remove(*source_node_id) {
            // Insert at new location
            if app.tree_root.insert_at_position(dnd.target, dnd.position, node) {
                moved_any = true;
            }
        }
    }

    if moved_any {
        // Only repack if we moved images (not just directories)
        if moved_contains_images(&app.tree_root, &dnd.source) {
            app.atlas = None;
            app.atlas_texture = None;
            app.pack_atlas();
        }
        app.status_message = "Moved items".into();
    }
}

fn moved_contains_images(root: &TreeNode, node_ids: &[NodeId]) -> bool {
    fn check_node(node: &TreeNode, target_ids: &[NodeId]) -> bool {
        match node {
            TreeNode::Directory(dir) => {
                if target_ids.contains(&dir.id) {
                    return contains_images_recursive(node);
                }
                for child in &dir.children {
                    if check_node(child, target_ids) {
                        return true;
                    }
                }
                false
            }
            TreeNode::Image(img) => target_ids.contains(&img.id),
        }
    }

    fn contains_images_recursive(node: &TreeNode) -> bool {
        match node {
            TreeNode::Directory(dir) => dir.children.iter().any(contains_images_recursive),
            TreeNode::Image(_) => true,
        }
    }

    check_node(root, node_ids)
}

pub(super) fn render_create_directory_modal(app: &mut ZephyrApp, ctx: &egui::Context) {
    if !app.show_create_dir_modal {
        return;
    }

    egui::Window::new("New Directory")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label("Enter directory name:");
            let response = ui.text_edit_singleline(&mut app.new_dir_name);

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                app.finalize_create_directory();
            } else if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.show_create_dir_modal = false;
                app.pending_parent_id = None;
            }

            if !response.has_focus() {
                response.request_focus();
            }

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    app.finalize_create_directory();
                }
                if ui.button("Cancel").clicked() {
                    app.show_create_dir_modal = false;
                    app.pending_parent_id = None;
                }
            });
        });
}

pub(super) fn render_clear_confirmation_modal(app: &mut ZephyrApp, ctx: &egui::Context) {
    if !app.show_clear_confirmation_modal {
        return;
    }

    egui::Window::new("Clear All")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label("Are you sure you want to clear everything?");
            ui.label("This will remove all images and directories from the project.");
            ui.label("This does not delete them from disk");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Clear All").clicked() {
                    super::file_ops::clear(app);
                    app.show_clear_confirmation_modal = false;
                }

                if ui.button("Cancel").clicked() {
                    app.show_clear_confirmation_modal = false;
                }
            });
        });
}

pub(super) fn render_rename_modal(app: &mut ZephyrApp, ctx: &egui::Context) {
    if app.renaming_node_id.is_none() {
        return;
    }

    egui::Window::new("Rename")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label("Enter new name:");
            let response = ui.text_edit_singleline(&mut app.rename_buffer);

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                app.finalize_rename();
            }

            if !response.has_focus() {
                response.request_focus();
            }

            ui.horizontal(|ui| {
                if ui.button("Rename").clicked() {
                    app.finalize_rename();
                }
                if ui.button("Cancel").clicked() {
                    app.renaming_node_id = None;
                }
            });
        });
}

fn node_ids_to_sprite_names(tree_root: &TreeNode, node_ids: &[NodeId]) -> Option<Vec<String>> {
    if node_ids.is_empty() {
        return None;
    }

    // Collect names from every node in the dragged set, preserving tree order
    // and deduplicating (a directory and one of its children could both be selected).
    let mut names: Vec<String> = Vec::new();
    for &id in node_ids {
        if id == 0 {
            continue;
        }
        collect_names_for_node(tree_root, id, &mut names);
    }

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    names.retain(|n| seen.insert(n.clone()));

    if names.is_empty() { None } else { Some(names) }
}

fn collect_names_for_node(tree_root: &TreeNode, target_id: NodeId, out: &mut Vec<String>) {
    if let Some(img) = tree_root.iter_images().find(|img| img.id == target_id) {
        out.push(img.image.name.clone());
        return;
    }

    fn visit(node: &TreeNode, target_id: NodeId, out: &mut Vec<String>) -> bool {
        match node {
            TreeNode::Directory(dir) => {
                if dir.id == target_id {
                    for child in &dir.children {
                        if let TreeNode::Image(img) = child {
                            out.push(img.image.name.clone());
                        }
                    }
                    return true;
                }
                for child in &dir.children {
                    if visit(child, target_id, out) {
                        return true;
                    }
                }
                false
            }
            TreeNode::Image(_) => false,
        }
    }

    visit(tree_root, target_id, out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Directory, ImageNode, SourceImage, TreeNode};

    fn make_image(id: NodeId, name: &str) -> TreeNode {
        TreeNode::Image(ImageNode {
            id,
            image: SourceImage {
                name: name.to_string(),
                data: image::RgbaImage::new(1, 1),
                width: 1,
                height: 1,
                path: None,
                source_width: 1,
                source_height: 1,
                trim_offset_x: 0,
                trim_offset_y: 0,
                node_id: 0,
            },
        })
    }

    fn make_dir(id: NodeId, children: Vec<TreeNode>) -> TreeNode {
        TreeNode::Directory(Directory { id, name: format!("dir_{id}"), children })
    }

    // Root
    //   dir_1/
    //     a (id 2)
    //     b (id 3)
    //   c (id 4)
    fn test_tree() -> TreeNode {
        make_dir(0, vec![make_dir(1, vec![make_image(2, "a"), make_image(3, "b")]), make_image(4, "c")])
    }

    #[test]
    fn single_image_node() {
        let tree = test_tree();
        assert_eq!(node_ids_to_sprite_names(&tree, &[2]), Some(vec!["a".to_string()]));
    }

    #[test]
    fn single_directory_node_collects_direct_children() {
        let tree = test_tree();
        assert_eq!(node_ids_to_sprite_names(&tree, &[1]), Some(vec!["a".to_string(), "b".to_string()]));
    }

    #[test]
    fn multiple_image_nodes() {
        let tree = test_tree();
        assert_eq!(node_ids_to_sprite_names(&tree, &[2, 4]), Some(vec!["a".to_string(), "c".to_string()]));
    }

    #[test]
    fn directory_and_child_both_selected_deduplicates() {
        let tree = test_tree();
        // dir_1 (id 1) contains "a" (id 2); selecting both must not produce "a" twice.
        let result = node_ids_to_sprite_names(&tree, &[1, 2]).unwrap();
        let count_a = result.iter().filter(|n| n.as_str() == "a").count();
        assert_eq!(count_a, 1, "\"a\" must appear exactly once");
        assert!(result.contains(&"b".to_string()));
    }

    #[test]
    fn root_id_is_skipped() {
        let tree = test_tree();
        // Dragging root alone -> None
        assert_eq!(node_ids_to_sprite_names(&tree, &[0]), None);
        // Dragging root together with a real image -> root is silently skipped
        assert_eq!(node_ids_to_sprite_names(&tree, &[0, 4]), Some(vec!["c".to_string()]));
    }

    #[test]
    fn empty_slice_returns_none() {
        let tree = test_tree();
        assert_eq!(node_ids_to_sprite_names(&tree, &[]), None);
    }
}
