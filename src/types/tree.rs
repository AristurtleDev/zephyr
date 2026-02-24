// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::project::SourceImage;

pub(crate) type NodeId = usize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum TreeNode {
    Directory(Directory),
    Image(ImageNode),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Directory {
    pub(crate) id: NodeId,
    pub(crate) name: String,
    pub(crate) children: Vec<TreeNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ImageNode {
    pub(crate) id: NodeId,
    pub(crate) image: SourceImage,
}

impl TreeNode {
    pub(crate) const fn id(&self) -> NodeId {
        match self {
            TreeNode::Directory(dir) => dir.id,
            TreeNode::Image(img) => img.id,
        }
    }

    pub(crate) fn iter_images(&self) -> TreeImageIterator<'_> {
        TreeImageIterator::new(self)
    }

    pub(crate) fn collect_images(&self) -> Vec<SourceImage> {
        self.iter_images()
            .map(|img_node| {
                let mut image = img_node.image.clone();
                image.node_id = img_node.id;
                image
            })
            .collect()
    }

    pub(crate) fn reload_images(&mut self) -> (usize, usize) {
        let mut loaded = 0;
        let mut failed = 0;

        match self {
            TreeNode::Directory(dir) => {
                for child in &mut dir.children {
                    let (l, f) = child.reload_images();
                    loaded += l;
                    failed += f;
                }
            }
            TreeNode::Image(img_node) => {
                if let Some(path) = &img_node.image.path {
                    match crate::loader::load_image(path) {
                        Ok(loaded_img) => {
                            img_node.image = loaded_img;
                            loaded += 1;
                        }
                        Err(e) => {
                            eprintln!("Failed to reload image {:?}: {}", path, e);
                            failed += 1;
                        }
                    }
                } else {
                    failed += 1;
                }
            }
        }

        (loaded, failed)
    }

    pub(crate) fn remove(&mut self, id: NodeId) -> Option<TreeNode> {
        if let TreeNode::Directory(dir) = self {
            if let Some(index) = dir.children.iter().position(|n| n.id() == id) {
                Some(dir.children.remove(index))
            } else {
                for child in &mut dir.children {
                    if let Some(removed) = child.remove(id) {
                        return Some(removed);
                    }
                }
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn insert(&mut self, parent_id: NodeId, node: TreeNode) -> bool {
        if let TreeNode::Directory(dir) = self {
            if dir.id == parent_id {
                dir.children.push(node);
                return true;
            } else {
                for child in &mut dir.children {
                    if child.insert(parent_id, node.clone()) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(crate) fn insert_at_position(&mut self, target_id: NodeId, position: egui_ltreeview::DirPosition<NodeId>, node: TreeNode) -> bool {
        use egui_ltreeview::DirPosition;

        match position {
            DirPosition::First => self.insert_as_first_child(target_id, node),
            DirPosition::Last => self.insert(target_id, node),
            DirPosition::Before(sibling_id) => self.insert_before_sibling(sibling_id, node),
            DirPosition::After(sibling_id) => self.insert_after_sibling(sibling_id, node),
        }
    }

    fn insert_as_first_child(&mut self, parent_id: NodeId, node: TreeNode) -> bool {
        if let TreeNode::Directory(dir) = self {
            if dir.id == parent_id {
                dir.children.insert(0, node);
                return true;
            }
            for child in &mut dir.children {
                if child.insert_as_first_child(parent_id, node.clone()) {
                    return true;
                }
            }
        }
        false
    }

    fn insert_before_sibling(&mut self, sibling_id: NodeId, node: TreeNode) -> bool {
        if let TreeNode::Directory(dir) = self {
            if let Some(pos) = dir.children.iter().position(|c| c.id() == sibling_id) {
                dir.children.insert(pos, node);
                return true;
            }
            for child in &mut dir.children {
                if child.insert_before_sibling(sibling_id, node.clone()) {
                    return true;
                }
            }
        }
        false
    }

    fn insert_after_sibling(&mut self, sibling_id: NodeId, node: TreeNode) -> bool {
        if let TreeNode::Directory(dir) = self {
            if let Some(pos) = dir.children.iter().position(|c| c.id() == sibling_id) {
                dir.children.insert(pos + 1, node);
                return true;
            }
            for child in &mut dir.children {
                if child.insert_after_sibling(sibling_id, node.clone()) {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn relativize_paths(&mut self, base: &Path) {
        match self {
            TreeNode::Directory(dir) => {
                for child in &mut dir.children {
                    child.relativize_paths(base);
                }
            }
            TreeNode::Image(img_node) => {
                if let Some(abs_path) = &img_node.image.path {
                    if let Ok(rel_path) = abs_path.strip_prefix(base) {
                        img_node.image.path = Some(rel_path.to_path_buf());
                    }
                }
            }
        }
    }

    pub(crate) fn absolutize_paths(&mut self, base: &Path) {
        match self {
            TreeNode::Directory(dir) => {
                for child in &mut dir.children {
                    child.absolutize_paths(base);
                }
            }
            TreeNode::Image(img_node) => {
                if let Some(rel_path) = &img_node.image.path {
                    if rel_path.is_relative() {
                        img_node.image.path = Some(base.join(rel_path));
                    }
                }
            }
        }
    }

    pub(crate) fn node_name(&self, id: NodeId) -> Option<&str> {
        match self {
            TreeNode::Directory(dir) => {
                if dir.id == id {
                    return Some(&dir.name);
                }
                for child in &dir.children {
                    if let Some(name) = child.node_name(id) {
                        return Some(name);
                    }
                }
                None
            }
            TreeNode::Image(img) => {
                if img.id == id {
                    Some(&img.image.name)
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn rename_node(&mut self, id: NodeId, new_name: String) -> bool {
        match self {
            TreeNode::Directory(dir) => {
                if dir.id == id {
                    dir.name = new_name;
                    return true;
                }
                for child in &mut dir.children {
                    if child.rename_node(id, new_name.clone()) {
                        return true;
                    }
                }
                false
            }
            TreeNode::Image(img) => {
                if img.id == id {
                    img.image.name = new_name;
                    true
                } else {
                    false
                }
            }
        }
    }
}

pub(crate) struct TreeImageIterator<'a> {
    stack: Vec<&'a TreeNode>,
}

impl<'a> TreeImageIterator<'a> {
    fn new(root: &'a TreeNode) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for TreeImageIterator<'a> {
    type Item = &'a ImageNode;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            match node {
                TreeNode::Directory(dir) => {
                    // Push children in reverse so the leftmost child is popped first.
                    self.stack.extend(dir.children.iter().rev());
                }
                TreeNode::Image(img) => {
                    return Some(img);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::project::SourceImage;
    use super::*;

    fn make_source_image(name: &str) -> SourceImage {
        SourceImage {
            data: image::RgbaImage::new(1, 1),
            width: 1,
            height: 1,
            name: name.to_string(),
            path: None,
            source_width: 1,
            source_height: 1,
            trim_offset_x: 0,
            trim_offset_y: 0,
            node_id: 0,
        }
    }

    fn make_image_node(id: NodeId, name: &str) -> TreeNode {
        TreeNode::Image(ImageNode { id, image: make_source_image(name) })
    }

    fn make_dir(id: NodeId, name: &str, children: Vec<TreeNode>) -> TreeNode {
        TreeNode::Directory(Directory { id, name: name.to_string(), children })
    }

    #[test]
    fn node_name_returns_root_directory_name() {
        let root = make_dir(0, "root", vec![]);
        assert_eq!(root.node_name(0), Some("root"));
    }

    #[test]
    fn node_name_returns_none_for_missing_id() {
        let root = make_dir(0, "root", vec![]);
        assert_eq!(root.node_name(99), None);
    }

    #[test]
    fn node_name_finds_nested_image() {
        let root = make_dir(0, "root", vec![make_image_node(1, "hero.png")]);
        assert_eq!(root.node_name(1), Some("hero.png"));
    }

    #[test]
    fn node_name_finds_nested_directory() {
        let root = make_dir(0, "root", vec![make_dir(1, "sprites", vec![])]);
        assert_eq!(root.node_name(1), Some("sprites"));
    }

    #[test]
    fn rename_node_updates_directory_name() {
        let mut root = make_dir(0, "root", vec![make_dir(1, "sprites", vec![])]);
        assert!(root.rename_node(1, "textures".to_string()));
        assert_eq!(root.node_name(1), Some("textures"));
    }

    #[test]
    fn rename_node_updates_image_name() {
        let mut root = make_dir(0, "root", vec![make_image_node(1, "hero.png")]);
        assert!(root.rename_node(1, "hero_v2.png".to_string()));
        assert_eq!(root.node_name(1), Some("hero_v2.png"));
    }

    #[test]
    fn rename_node_returns_false_for_missing_id() {
        let mut root = make_dir(0, "root", vec![]);
        assert!(!root.rename_node(99, "new".to_string()));
    }

    #[test]
    fn rename_node_does_not_affect_other_nodes() {
        let mut root = make_dir(0, "root", vec![make_image_node(1, "a.png"), make_image_node(2, "b.png")]);
        assert!(root.rename_node(1, "renamed.png".to_string()));
        assert_eq!(root.node_name(2), Some("b.png"));
    }
}
