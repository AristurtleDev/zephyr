// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use std::collections::HashMap;
use std::path::PathBuf;

use image::RgbaImage;
use serde::{Deserialize, Serialize};

use super::animation::Animation;
use super::settings::PackSettings;
use super::sprite::SpriteProperties;
use super::tree::TreeNode;

// Increment this only when the .zephyr file schema changes in a
// backwards-incompatible way. It is intentionally decoupled from the
// application version in Cargo.toml.
pub(crate) const FILE_FORMAT_VERSION: &str = "1.0";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SourceImage {
    #[serde(skip)]
    #[serde(default = "default_rgba_image")]
    pub(crate) data: RgbaImage,

    #[serde(skip)]
    pub(crate) width: u32,

    #[serde(skip)]
    pub(crate) height: u32,

    #[serde(skip)]
    pub(crate) name: String,

    pub(crate) path: Option<PathBuf>,

    // Original frame dimensions before any trim was applied. Equal to width/height when
    // no trim is active.
    #[serde(skip)]
    pub(crate) source_width: u32,
    #[serde(skip)]
    pub(crate) source_height: u32,

    // Top-left offset of the packed (possibly trimmed) region within the original frame.
    // Both are zero when no trim is active.
    #[serde(skip)]
    pub(crate) trim_offset_x: u32,
    #[serde(skip)]
    pub(crate) trim_offset_y: u32,
}

fn default_rgba_image() -> RgbaImage {
    RgbaImage::new(1, 1)
}

pub(crate) struct PackedAtlas {
    pub(crate) texture: RgbaImage,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) placements: Vec<Placement>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct Placement {
    pub(crate) name: String,
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) source_width: u32,
    pub(crate) source_height: u32,
    pub(crate) trim_offset_x: u32,
    pub(crate) trim_offset_y: u32,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ZephyrProject {
    pub(crate) version: String,
    pub(crate) settings: PackSettings,
    pub(crate) tree_root: TreeNode,
    pub(crate) sprite_properties: HashMap<String, SpriteProperties>,
    pub(crate) animations: Vec<Animation>,
}

impl ZephyrProject {
    pub(crate) fn new(settings: PackSettings, tree_root: &TreeNode, sprite_properties: HashMap<String, SpriteProperties>, animations: Vec<Animation>) -> Self {
        Self {
            version: FILE_FORMAT_VERSION.to_string(),
            settings,
            tree_root: tree_root.clone(),
            sprite_properties,
            animations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::animation::AnimationFrame;
    use super::super::enums::{AlphaProcessing, PackingAlgorithm, PixelFormat, PlaybackDirection, ShelfSortBy, SortOrder, TextureFormat, TrimMode};
    use super::super::hitbox::HitboxShape;
    use super::super::sprite::PivotPoint;
    use super::super::tree::{Directory, ImageNode};
    use super::*;

    #[test]
    fn json_round_trip() {
        // Build a project with two image nodes inside a sub-directory,
        // non-default pack settings, one animation with two frames, and
        // sprite properties containing a circle hitbox.
        let children = vec![
            TreeNode::Image(ImageNode {
                id: 1,
                image: SourceImage {
                    data: default_rgba_image(),
                    width: 32,
                    height: 32,
                    name: "sprite_a".to_string(),
                    path: None,
                    source_width: 32,
                    source_height: 32,
                    trim_offset_x: 0,
                    trim_offset_y: 0,
                },
            }),
            TreeNode::Image(ImageNode {
                id: 2,
                image: SourceImage {
                    data: default_rgba_image(),
                    width: 64,
                    height: 64,
                    name: "sprite_b".to_string(),
                    path: None,
                    source_width: 64,
                    source_height: 64,
                    trim_offset_x: 0,
                    trim_offset_y: 0,
                },
            }),
        ];

        let sub_dir = TreeNode::Directory(Directory {
            id: 3,
            name: "characters".to_string(),
            children,
        });

        let tree_root = TreeNode::Directory(Directory {
            id: 0,
            name: "root".to_string(),
            children: vec![sub_dir],
        });

        let settings = PackSettings {
            shape_padding: 4,
            border_padding: 8,
            extrude: 2,
            algorithm: PackingAlgorithm::MaxRects,
            force_square: false,
            grid_alignment: 0,
            trim_mode: crate::types::enums::TrimMode::Trim,
            shelf_sort_by: ShelfSortBy::Name,
            shelf_sort_order: SortOrder::Ascending,
            texture_format: TextureFormat::Png,
            pixel_format: PixelFormat::RGBA8,
            alpha_processing: AlphaProcessing::Premultiplied,
            png_compression_level: 6,
            jpeg_quality: 90,
            compression_quality: 75,
            generate_mipmaps: true,
        };

        let mut sprite_properties = HashMap::new();
        sprite_properties.insert(
            "sprite_a".to_string(),
            SpriteProperties {
                pivot_enabled: true,
                pivot: PivotPoint::default(),
                hitbox_enabled: true,
                hitbox: HitboxShape::Circle { cx: 16.0, cy: 16.0, radius: 12.0 },
            },
        );

        let mut anim = Animation::new("idle");
        anim.frames.push(AnimationFrame::new("sprite_a"));
        anim.frames.push(AnimationFrame {
            sprite_name: "sprite_b".to_string(),
            delay_ms: 200,
        });

        let project = ZephyrProject::new(settings, &tree_root, sprite_properties, vec![anim]);

        // Serialize then deserialize.
        let json = serde_json::to_string_pretty(&project).unwrap();
        let recovered: ZephyrProject = serde_json::from_str(&json).unwrap();

        // Version
        assert_eq!(recovered.version, FILE_FORMAT_VERSION);

        // Settings
        assert_eq!(recovered.settings.shape_padding, 4);
        assert_eq!(recovered.settings.border_padding, 8);
        assert_eq!(recovered.settings.extrude, 2);
        assert!(matches!(recovered.settings.algorithm, PackingAlgorithm::MaxRects));
        assert!(matches!(recovered.settings.shelf_sort_by, ShelfSortBy::Name));
        assert!(matches!(recovered.settings.shelf_sort_order, SortOrder::Ascending));
        assert!(!recovered.settings.force_square);
        assert!(matches!(recovered.settings.trim_mode, TrimMode::Trim));
        assert!(matches!(recovered.settings.alpha_processing, AlphaProcessing::Premultiplied));
        assert_eq!(recovered.settings.png_compression_level, 6);
        assert_eq!(recovered.settings.jpeg_quality, 90);
        assert_eq!(recovered.settings.compression_quality, 75);
        assert!(recovered.settings.generate_mipmaps);

        // Tree structure
        let TreeNode::Directory(root) = &recovered.tree_root else {
            panic!("tree root must be a Directory");
        };
        assert_eq!(root.id, 0);
        assert_eq!(root.name, "root");
        assert_eq!(root.children.len(), 1);
        let TreeNode::Directory(sub) = &root.children[0] else {
            panic!("first child must be a Directory");
        };
        assert_eq!(sub.name, "characters");
        assert_eq!(sub.children.len(), 2);

        // Animations
        assert_eq!(recovered.animations.len(), 1);
        let rec_anim = &recovered.animations[0];
        assert_eq!(rec_anim.name, "idle");
        assert_eq!(rec_anim.frames.len(), 2);
        assert_eq!(rec_anim.frames[0].sprite_name, "sprite_a");
        assert_eq!(rec_anim.frames[1].delay_ms, 200);
        assert!(
            matches!(rec_anim.direction, PlaybackDirection::Forward),
            "new animations default to Forward direction",
        );
        assert!(rec_anim.loop_enabled, "new animations default to looping");

        // Sprite properties
        let props = recovered.sprite_properties.get("sprite_a").expect("sprite_a properties must round-trip");
        assert!(props.pivot_enabled);
        assert!(props.hitbox_enabled);
        assert!(
            matches!(props.hitbox, HitboxShape::Circle { cx, cy, radius }
                if cx == 16.0 && cy == 16.0 && radius == 12.0),
            "circle hitbox parameters must round-trip exactly",
        );
    }

    #[test]
    fn minimal_fixture_deserializes() {
        // This test is an early warning for schema changes.
        // If the ZephyrProject schema changes unintentionally, deserialization
        // of this fixture will fail.
        // If schema changes are deliberate, then tests/fixtures/minimal.zephyr
        // should be updated to match.
        let fixture = include_str!("../../tests/fixtures/minimal.zephyr");
        let project: ZephyrProject = serde_json::from_str(fixture).expect("tests/fixtures/minimal.zephyr must deserialize without error");
        assert_eq!(project.version, FILE_FORMAT_VERSION);
        let TreeNode::Directory(root) = &project.tree_root else {
            panic!("minimal fixture tree_root must be a Directory");
        };
        assert_eq!(root.name, "root");
        assert!(project.animations.is_empty());
        assert!(project.sprite_properties.is_empty());
    }
}
