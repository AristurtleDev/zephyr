// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use crate::types::{PackSettings, PackedAtlas, PackingAlgorithm, Placement, ShelfSortBy, SortOrder, SourceImage};
use image::RgbaImage;

// Rounds `value` up to the nearest multiple of `grid`. When `grid` is zero the
// value is returned unchanged (grid alignment disabled).
fn align_up(value: u32, grid: u32) -> u32 {
    if grid == 0 {
        return value;
    }
    ((value + grid - 1) / grid) * grid
}

fn find_optimal_size(images: &[SourceImage], settings: &PackSettings) -> Option<(u32, u32)> {
    // Always use power-of-2 sizes from 32 to 4096
    let sizes = [32, 64, 128, 256, 512, 1024, 2048, 4096];

    let mut best_size: Option<(u32, u32)> = None;
    let mut best_area = u32::MAX;

    // Try all square sizes
    for &size in &sizes {
        if try_pack(images, size, size, settings).is_some() {
            let area = size * size;
            if area < best_area {
                best_area = area;
                best_size = Some((size, size));
            }

            if settings.force_square {
                return best_size;
            }
        }
    }

    // If force_square is enabled, return the best square we found (or None)
    if settings.force_square {
        return best_size;
    }

    // Try rectangular sizes to find even better solution
    // Limit aspect ratio to 4:1 or 1:4 to avoid extremely thin atlases
    for &width in &sizes {
        for &height in &sizes {
            // Skip squares (already tried)
            if width == height {
                continue;
            }

            let aspect_ratio = if width > height {
                width as f32 / height as f32
            } else {
                height as f32 / width as f32
            };

            if aspect_ratio > 4.0 {
                continue;
            }

            // Try packing with this size
            if try_pack(images, width, height, settings).is_some() {
                let area = width * height;
                if area < best_area {
                    best_area = area;
                    best_size = Some((width, height));
                }
            }
        }
    }

    best_size
}

fn try_pack(images: &[SourceImage], atlas_width: u32, atlas_height: u32, settings: &PackSettings) -> Option<PackedAtlas> {
    match settings.algorithm {
        PackingAlgorithm::Shelf => pack_shelf(images, atlas_width, atlas_height, settings),
        PackingAlgorithm::MaxRects => pack_maxrects(images, atlas_width, atlas_height, settings),
    }
}

pub(crate) fn pack_images(images: &[SourceImage], settings: &PackSettings) -> Option<PackedAtlas> {
    if images.is_empty() {
        return None;
    }

    let (atlas_width, atlas_height) = find_optimal_size(images, settings)?;

    match settings.algorithm {
        PackingAlgorithm::Shelf => pack_shelf(images, atlas_width, atlas_height, settings),
        PackingAlgorithm::MaxRects => pack_maxrects(images, atlas_width, atlas_height, settings),
    }
}

fn pack_shelf(images: &[SourceImage], atlas_width: u32, atlas_height: u32, settings: &PackSettings) -> Option<PackedAtlas> {
    let mut sorted_indices: Vec<usize> = (0..images.len()).collect();
    let descending = settings.shelf_sort_order == SortOrder::Descending;
    sorted_indices.sort_by(|&a, &b| {
        let ordering = match settings.shelf_sort_by {
            ShelfSortBy::Best | ShelfSortBy::Height => images[a].height.cmp(&images[b].height),
            ShelfSortBy::Width => images[a].width.cmp(&images[b].width),
            ShelfSortBy::Circumference => {
                let c_a = images[a].width + images[a].height;
                let c_b = images[b].width + images[b].height;
                c_a.cmp(&c_b)
            }
            ShelfSortBy::Name => images[a].name.cmp(&images[b].name),
            ShelfSortBy::Directory => {
                let dir_a = images[a].path.as_ref().and_then(|p| p.parent()).and_then(|p| p.to_str()).unwrap_or("");
                let dir_b = images[b].path.as_ref().and_then(|p| p.parent()).and_then(|p| p.to_str()).unwrap_or("");
                dir_a.cmp(dir_b)
            }
        };
        if descending { ordering.reverse() } else { ordering }
    });

    let mut placements = Vec::new();

    let mut current_x = settings.border_padding;
    let mut current_y = settings.border_padding;
    let mut shelf_height = 0u32;

    // Calculate available space (excluding border padding on both sides)
    let available_width = atlas_width.saturating_sub(settings.border_padding * 2);
    let available_height = atlas_height.saturating_sub(settings.border_padding * 2);

    for &idx in &sorted_indices {
        let img = &images[idx];

        // Snap the sprite's top-left to the grid. The sprite top-left is at
        // (current_x + extrude, current_y + extrude), so we snap those
        // coordinates up and compute how wide the slot actually is.
        let aligned_x = align_up(current_x + settings.extrude, settings.grid_alignment);
        let x_snap_gap = aligned_x.saturating_sub(current_x + settings.extrude);

        // Slot width includes the alignment gap consumed before the sprite
        let slot_width = x_snap_gap + settings.extrude * 2 + img.width + settings.shape_padding;
        let sprite_total_height = settings.extrude * 2 + img.height + settings.shape_padding;

        // Check if image fits on current shelf (relative to border padding)
        if current_x - settings.border_padding + slot_width > available_width {
            // Start new shelf
            current_x = settings.border_padding;
            current_y += shelf_height;

            // Recompute snap for the new cursor position
            let new_aligned_x = align_up(current_x + settings.extrude, settings.grid_alignment);
            let new_x_snap_gap = new_aligned_x.saturating_sub(current_x + settings.extrude);
            let new_slot_width = new_x_snap_gap + settings.extrude * 2 + img.width + settings.shape_padding;

            // Y snap gap for the new shelf row
            let aligned_y = align_up(current_y + settings.extrude, settings.grid_alignment);
            let y_snap_gap = aligned_y.saturating_sub(current_y + settings.extrude);

            // Check if new shelf fits in available height (including any y snap gap)
            if current_y - settings.border_padding + y_snap_gap + sprite_total_height > available_height {
                return None;
            }

            placements.push(Placement {
                name: img.name.clone(),
                x: new_aligned_x,
                y: aligned_y,
                width: img.width,
                height: img.height,
                source_width: img.source_width,
                source_height: img.source_height,
                trim_offset_x: img.trim_offset_x,
                trim_offset_y: img.trim_offset_y,
                node_id: img.node_id,
            });

            current_x += new_slot_width;
            // shelf_height must cover the y snap gap so the next shelf starts below this one
            shelf_height = y_snap_gap + sprite_total_height;
            continue;
        }

        let aligned_y = align_up(current_y + settings.extrude, settings.grid_alignment);
        let y_snap_gap = aligned_y.saturating_sub(current_y + settings.extrude);

        // Taller sprites placed on the current shelf can overflow the atlas
        // bottom even though they fit horizontally. Reject here for the same
        // reason the new-shelf branch does.
        if current_y - settings.border_padding + y_snap_gap + sprite_total_height > available_height {
            return None;
        }

        placements.push(Placement {
            name: img.name.clone(),
            x: aligned_x,
            y: aligned_y,
            width: img.width,
            height: img.height,
            source_width: img.source_width,
            source_height: img.source_height,
            trim_offset_x: img.trim_offset_x,
            trim_offset_y: img.trim_offset_y,
            node_id: img.node_id,
        });

        current_x += slot_width;
        shelf_height = shelf_height.max(y_snap_gap + sprite_total_height);
    }

    Some(PackedAtlas {
        texture: RgbaImage::new(atlas_width, atlas_height),
        width: atlas_width,
        height: atlas_height,
        placements,
    })
}

fn pack_maxrects(images: &[SourceImage], atlas_width: u32, atlas_height: u32, settings: &PackSettings) -> Option<PackedAtlas> {
    #[derive(Clone, Copy, Debug)]
    struct Rect {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    }

    impl Rect {
        fn intersects(&self, other: &Rect) -> bool {
            self.x < other.x + other.width && self.x + self.width > other.x && self.y < other.y + other.height && self.y + self.height > other.y
        }

        fn contains(&self, other: &Rect) -> bool {
            other.x >= self.x && other.y >= self.y && other.x + other.width <= self.x + self.width && other.y + other.height <= self.y + self.height
        }
    }

    let mut sorted_indices: Vec<usize> = (0..images.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        let area_a = images[a].width * images[a].height;
        let area_b = images[b].width * images[b].height;
        area_b.cmp(&area_a)
    });

    let mut placements = Vec::new();

    // Calculate available space (excluding border padding)
    let available_width = atlas_width.saturating_sub(settings.border_padding * 2);
    let available_height = atlas_height.saturating_sub(settings.border_padding * 2);

    // Initialize with the available rectangle (offset by border_padding)
    let mut free_rects = vec![Rect {
        x: settings.border_padding,
        y: settings.border_padding,
        width: available_width,
        height: available_height,
    }];

    for &idx in &sorted_indices {
        let img = &images[idx];

        // Base size without alignment gaps
        let base_width = settings.extrude * 2 + img.width + settings.shape_padding;
        let base_height = settings.extrude * 2 + img.height + settings.shape_padding;

        // Find best free rectangle using Best Short Side Fit (BSSF).
        // When grid alignment is active each candidate position is snapped up
        // to the grid within its free rect, which may require extra space.
        let mut best_rect_idx = None;
        let mut best_short_side_fit = u32::MAX;
        let mut best_long_side_fit = u32::MAX;

        for (i, rect) in free_rects.iter().enumerate() {
            // Snap the sprite top-left to the grid within this free rect
            let sprite_x = align_up(rect.x + settings.extrude, settings.grid_alignment);
            let sprite_y = align_up(rect.y + settings.extrude, settings.grid_alignment);
            let x_gap = sprite_x.saturating_sub(rect.x + settings.extrude);
            let y_gap = sprite_y.saturating_sub(rect.y + settings.extrude);

            let needed_width = x_gap + base_width;
            let needed_height = y_gap + base_height;

            if needed_width <= rect.width && needed_height <= rect.height {
                let leftover_x = rect.width - needed_width;
                let leftover_y = rect.height - needed_height;
                let short_side_fit = leftover_x.min(leftover_y);
                let long_side_fit = leftover_x.max(leftover_y);

                if short_side_fit < best_short_side_fit || (short_side_fit == best_short_side_fit && long_side_fit < best_long_side_fit) {
                    best_rect_idx = Some(i);
                    best_short_side_fit = short_side_fit;
                    best_long_side_fit = long_side_fit;
                }
            }
        }

        if let Some(rect_idx) = best_rect_idx {
            let used_rect = free_rects[rect_idx];

            // Compute grid-snapped sprite position within the chosen free rect
            let sprite_x = align_up(used_rect.x + settings.extrude, settings.grid_alignment);
            let sprite_y = align_up(used_rect.y + settings.extrude, settings.grid_alignment);
            let x_gap = sprite_x.saturating_sub(used_rect.x + settings.extrude);
            let y_gap = sprite_y.saturating_sub(used_rect.y + settings.extrude);

            // The placed rect covers the alignment gap plus the full sprite slot
            let placed = Rect {
                x: used_rect.x,
                y: used_rect.y,
                width: x_gap + base_width,
                height: y_gap + base_height,
            };

            placements.push(Placement {
                name: img.name.clone(),
                x: sprite_x,
                y: sprite_y,
                width: img.width,
                height: img.height,
                source_width: img.source_width,
                source_height: img.source_height,
                trim_offset_x: img.trim_offset_x,
                trim_offset_y: img.trim_offset_y,
                node_id: img.node_id,
            });

            // Split all free rectangles that intersect with the placed rectangle
            let mut new_rects = Vec::new();

            for rect in &free_rects {
                if rect.intersects(&placed) {
                    // Generate new rectangles from the intersection
                    // Left side
                    if rect.x < placed.x {
                        new_rects.push(Rect {
                            x: rect.x,
                            y: rect.y,
                            width: placed.x - rect.x,
                            height: rect.height,
                        });
                    }

                    // Right side
                    if rect.x + rect.width > placed.x + placed.width {
                        new_rects.push(Rect {
                            x: placed.x + placed.width,
                            y: rect.y,
                            width: rect.x + rect.width - (placed.x + placed.width),
                            height: rect.height,
                        });
                    }

                    // Bottom side
                    if rect.y < placed.y {
                        new_rects.push(Rect {
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: placed.y - rect.y,
                        });
                    }

                    // Top side
                    if rect.y + rect.height > placed.y + placed.height {
                        new_rects.push(Rect {
                            x: rect.x,
                            y: placed.y + placed.height,
                            width: rect.width,
                            height: rect.y + rect.height - (placed.y + placed.height),
                        });
                    }
                }
            }

            // Remove all rectangles that intersect with the placed rectangle
            free_rects.retain(|rect| !rect.intersects(&placed));

            // Add the new split rectangles
            for new_rect in new_rects {
                if new_rect.width > 0 && new_rect.height > 0 {
                    free_rects.push(new_rect);
                }
            }

            // Prune redundant rectangles (remove those contained within others)
            let mut i = 0;
            while i < free_rects.len() {
                let mut j = i + 1;
                let mut removed = false;
                while j < free_rects.len() {
                    if free_rects[j].contains(&free_rects[i]) {
                        // rect[i] is contained in rect[j], remove rect[i]
                        free_rects.remove(i);
                        removed = true;
                        break;
                    } else if free_rects[i].contains(&free_rects[j]) {
                        // rect[j] is contained in rect[i], remove rect[j]
                        free_rects.remove(j);
                        // Don't increment j, check the new element at this position
                    } else {
                        j += 1;
                    }
                }
                if !removed {
                    i += 1;
                }
            }
        } else {
            // Image doesn't fit
            return None;
        }
    }

    Some(PackedAtlas {
        texture: RgbaImage::new(atlas_width, atlas_height),
        width: atlas_width,
        height: atlas_height,
        placements,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placements_overlap(a: &Placement, b: &Placement) -> bool {
        !(a.x + a.width <= b.x || b.x + b.width <= a.x || a.y + a.height <= b.y || b.y + b.height <= a.y)
    }

    fn make_source_image(name: &str, size: u32) -> SourceImage {
        SourceImage {
            data: RgbaImage::new(size, size),
            width: size,
            height: size,
            name: name.to_string(),
            path: None,
            source_width: size,
            source_height: size,
            trim_offset_x: 0,
            trim_offset_y: 0,
            node_id: 0,
        }
    }

    fn tight_settings(alg: PackingAlgorithm) -> PackSettings {
        PackSettings {
            shape_padding: 0,
            border_padding: 0,
            extrude: 0,
            algorithm: alg,
            ..PackSettings::default()
        }
    }

    fn make_sized_source_image(name: &str, width: u32, height: u32) -> SourceImage {
        SourceImage {
            data: RgbaImage::new(width, height),
            width,
            height,
            name: name.to_string(),
            path: None,
            source_width: width,
            source_height: height,
            trim_offset_x: 0,
            trim_offset_y: 0,
            node_id: 0,
        }
    }

    fn shelf_sort_settings(sort_by: ShelfSortBy, order: SortOrder) -> PackSettings {
        PackSettings {
            shape_padding: 0,
            border_padding: 0,
            extrude: 0,
            algorithm: PackingAlgorithm::Shelf,
            shelf_sort_by: sort_by,
            shelf_sort_order: order,
            ..PackSettings::default()
        }
    }

    #[test]
    fn shelf_tall_image_on_current_shelf_returns_none_when_height_overflows() {
        // A short sprite occupies the start of the first shelf. A tall sprite then fits
        // horizontally on the same shelf but its bottom would exceed the atlas height minus
        // border padding. Before the fix pack_shelf returned Some with an out-of-bounds
        // placement; now it must return None so find_optimal_size selects a larger atlas.
        //
        // Atlas 100x60, border_padding 4: available_height = 52, available_width = 92.
        // "short" at (4, 4), current_x advances to 34, shelf_height = 10.
        // "tall" (height 53): slot fits horizontally (34 - 4 + 30 = 60 <= 92) but
        // bottom = 4 + 53 = 57 exceeds atlas_height - border_padding = 56.
        let settings = PackSettings {
            shape_padding: 0,
            border_padding: 4,
            extrude: 0,
            algorithm: PackingAlgorithm::Shelf,
            shelf_sort_by: ShelfSortBy::Height,
            shelf_sort_order: SortOrder::Ascending,
            ..PackSettings::default()
        };
        let images = vec![
            make_sized_source_image("short", 30, 10),
            make_sized_source_image("tall", 30, 53),
        ];
        assert!(try_pack(&images, 100, 60, &settings).is_none());
    }

    #[test]
    fn pack_images_empty_returns_none() {
        assert!(pack_images(&[], &PackSettings::default()).is_none());
    }

    #[test]
    fn pack_images_oversized_returns_none() {
        // 5000x5000 exceeds the maximum atlas size of 4096x4096.
        let img = make_source_image("big", 5000);
        assert!(pack_images(&[img], &PackSettings::default()).is_none());
    }

    #[test]
    fn try_pack_image_too_large_for_given_atlas_returns_none() {
        // 512x512 image cannot fit in a 256x256 atlas.
        let img = make_source_image("oversized", 512);
        let settings = tight_settings(PackingAlgorithm::Shelf);
        assert!(try_pack(&[img], 256, 256, &settings).is_none());
    }

    #[test]
    fn shelf_identical_squares_no_overlap() {
        let settings = tight_settings(PackingAlgorithm::Shelf);
        let images: Vec<SourceImage> = (0..9).map(|i| make_source_image(&format!("img{i}"), 32)).collect();

        let atlas = try_pack(&images, 256, 256, &settings).expect("nine 32x32 sprites must fit in a 256x256 atlas");

        for i in 0..atlas.placements.len() {
            for j in (i + 1)..atlas.placements.len() {
                let a = &atlas.placements[i];
                let b = &atlas.placements[j];
                assert!(!placements_overlap(a, b), "Shelf: placements '{}' and '{}' overlap", a.name, b.name,);
            }
        }
    }

    #[test]
    fn shelf_placements_within_atlas_bounds() {
        let settings = tight_settings(PackingAlgorithm::Shelf);
        let images: Vec<SourceImage> = (0..9).map(|i| make_source_image(&format!("img{i}"), 32)).collect();

        let atlas = try_pack(&images, 256, 256, &settings).expect("nine 32x32 sprites must fit in a 256x256 atlas");

        for p in &atlas.placements {
            assert!(
                p.x + p.width <= atlas.width,
                "Shelf: placement '{}' overflows atlas width ({} + {} > {})",
                p.name,
                p.x,
                p.width,
                atlas.width,
            );
            assert!(
                p.y + p.height <= atlas.height,
                "Shelf: placement '{}' overflows atlas height ({} + {} > {})",
                p.name,
                p.y,
                p.height,
                atlas.height,
            );
        }
    }

    #[test]
    fn shelf_packing_is_deterministic() {
        let settings = tight_settings(PackingAlgorithm::Shelf);
        let images: Vec<SourceImage> = (0..9).map(|i| make_source_image(&format!("img{i}"), 32)).collect();

        let first = try_pack(&images, 256, 256, &settings).expect("first pack must succeed");
        let second = try_pack(&images, 256, 256, &settings).expect("second pack must succeed");

        assert_eq!(first.placements.len(), second.placements.len());
        for (a, b) in first.placements.iter().zip(second.placements.iter()) {
            assert_eq!(a.name, b.name, "Shelf determinism: names differ");
            assert_eq!(a.x, b.x, "Shelf determinism: x positions differ for '{}'", a.name);
            assert_eq!(a.y, b.y, "Shelf determinism: y positions differ for '{}'", a.name);
        }
    }

    #[test]
    fn maxrects_identical_squares_no_overlap() {
        let settings = tight_settings(PackingAlgorithm::MaxRects);
        let images: Vec<SourceImage> = (0..9).map(|i| make_source_image(&format!("img{i}"), 32)).collect();

        let atlas = try_pack(&images, 256, 256, &settings).expect("nine 32x32 sprites must fit in a 256x256 atlas");

        for i in 0..atlas.placements.len() {
            for j in (i + 1)..atlas.placements.len() {
                let a = &atlas.placements[i];
                let b = &atlas.placements[j];
                assert!(!placements_overlap(a, b), "MaxRects: placements '{}' and '{}' overlap", a.name, b.name,);
            }
        }
    }

    #[test]
    fn maxrects_placements_within_atlas_bounds() {
        let settings = tight_settings(PackingAlgorithm::MaxRects);
        let images: Vec<SourceImage> = (0..9).map(|i| make_source_image(&format!("img{i}"), 32)).collect();

        let atlas = try_pack(&images, 256, 256, &settings).expect("nine 32x32 sprites must fit in a 256x256 atlas");

        for p in &atlas.placements {
            assert!(
                p.x + p.width <= atlas.width,
                "MaxRects: placement '{}' overflows atlas width ({} + {} > {})",
                p.name,
                p.x,
                p.width,
                atlas.width,
            );
            assert!(
                p.y + p.height <= atlas.height,
                "MaxRects: placement '{}' overflows atlas height ({} + {} > {})",
                p.name,
                p.y,
                p.height,
                atlas.height,
            );
        }
    }

    #[test]
    fn maxrects_packing_is_deterministic() {
        let settings = tight_settings(PackingAlgorithm::MaxRects);
        let images: Vec<SourceImage> = (0..9).map(|i| make_source_image(&format!("img{i}"), 32)).collect();

        let first = try_pack(&images, 256, 256, &settings).expect("first pack must succeed");
        let second = try_pack(&images, 256, 256, &settings).expect("second pack must succeed");

        assert_eq!(first.placements.len(), second.placements.len());
        for (a, b) in first.placements.iter().zip(second.placements.iter()) {
            assert_eq!(a.name, b.name, "MaxRects determinism: names differ");
            assert_eq!(a.x, b.x, "MaxRects determinism: x positions differ for '{}'", a.name);
            assert_eq!(a.y, b.y, "MaxRects determinism: y positions differ for '{}'", a.name);
        }
    }

    #[test]
    fn align_up_already_aligned_unchanged() {
        assert_eq!(align_up(0, 16), 0);
        assert_eq!(align_up(16, 16), 16);
        assert_eq!(align_up(32, 16), 32);
    }

    #[test]
    fn align_up_unaligned_rounds_up() {
        assert_eq!(align_up(1, 16), 16);
        assert_eq!(align_up(15, 16), 16);
        assert_eq!(align_up(17, 16), 32);
        assert_eq!(align_up(31, 16), 32);
    }

    #[test]
    fn align_up_grid_zero_is_noop() {
        assert_eq!(align_up(0, 0), 0);
        assert_eq!(align_up(17, 0), 17);
        assert_eq!(align_up(999, 0), 999);
    }

    #[test]
    fn align_up_grid_one_is_noop() {
        assert_eq!(align_up(0, 1), 0);
        assert_eq!(align_up(17, 1), 17);
        assert_eq!(align_up(999, 1), 999);
    }

    fn grid_settings(alg: PackingAlgorithm, grid: u32) -> PackSettings {
        PackSettings {
            shape_padding: 0,
            border_padding: 0,
            extrude: 0,
            algorithm: alg,
            grid_alignment: grid,
            ..PackSettings::default()
        }
    }

    fn assert_grid_aligned(placements: &[Placement], grid: u32) {
        for p in placements {
            assert_eq!(p.x % grid, 0, "placement '{}' x={} is not aligned to grid {}", p.name, p.x, grid,);
            assert_eq!(p.y % grid, 0, "placement '{}' y={} is not aligned to grid {}", p.name, p.y, grid,);
        }
    }

    #[test]
    fn shelf_grid_alignment_positions_are_grid_aligned() {
        let settings = grid_settings(PackingAlgorithm::Shelf, 16);
        // Use sprites whose natural positions would not be grid-aligned
        let images: Vec<SourceImage> = [10u32, 20, 15, 30, 5]
            .iter()
            .enumerate()
            .map(|(i, &s)| make_source_image(&format!("img{i}"), s))
            .collect();

        let atlas = try_pack(&images, 256, 256, &settings).expect("sprites must fit");

        assert_grid_aligned(&atlas.placements, 16);
    }

    #[test]
    fn shelf_grid_alignment_no_overlap() {
        let settings = grid_settings(PackingAlgorithm::Shelf, 16);
        let images: Vec<SourceImage> = [10u32, 20, 15, 30, 5]
            .iter()
            .enumerate()
            .map(|(i, &s)| make_source_image(&format!("img{i}"), s))
            .collect();

        let atlas = try_pack(&images, 256, 256, &settings).expect("sprites must fit");

        for i in 0..atlas.placements.len() {
            for j in (i + 1)..atlas.placements.len() {
                let a = &atlas.placements[i];
                let b = &atlas.placements[j];
                assert!(!placements_overlap(a, b), "Shelf grid: '{}' and '{}' overlap", a.name, b.name);
            }
        }
    }

    #[test]
    fn maxrects_grid_alignment_positions_are_grid_aligned() {
        let settings = grid_settings(PackingAlgorithm::MaxRects, 16);
        let images: Vec<SourceImage> = [10u32, 20, 15, 30, 5]
            .iter()
            .enumerate()
            .map(|(i, &s)| make_source_image(&format!("img{i}"), s))
            .collect();

        let atlas = try_pack(&images, 256, 256, &settings).expect("sprites must fit");

        assert_grid_aligned(&atlas.placements, 16);
    }

    #[test]
    fn maxrects_grid_alignment_no_overlap() {
        let settings = grid_settings(PackingAlgorithm::MaxRects, 16);
        let images: Vec<SourceImage> = [10u32, 20, 15, 30, 5]
            .iter()
            .enumerate()
            .map(|(i, &s)| make_source_image(&format!("img{i}"), s))
            .collect();

        let atlas = try_pack(&images, 256, 256, &settings).expect("sprites must fit");

        for i in 0..atlas.placements.len() {
            for j in (i + 1)..atlas.placements.len() {
                let a = &atlas.placements[i];
                let b = &atlas.placements[j];
                assert!(!placements_overlap(a, b), "MaxRects grid: '{}' and '{}' overlap", a.name, b.name);
            }
        }
    }

    #[test]
    fn shelf_name_sort_ascending_first_placement_is_alphabetically_first() {
        let settings = shelf_sort_settings(ShelfSortBy::Name, SortOrder::Ascending);
        let images = vec![
            make_sized_source_image("charlie", 32, 32),
            make_sized_source_image("alpha", 32, 32),
            make_sized_source_image("bravo", 32, 32),
        ];

        let atlas = try_pack(&images, 256, 256, &settings).expect("must fit");

        assert_eq!(atlas.placements[0].name, "alpha", "ascending name sort must place 'alpha' first");
    }

    #[test]
    fn shelf_name_sort_descending_first_placement_is_alphabetically_last() {
        let settings = shelf_sort_settings(ShelfSortBy::Name, SortOrder::Descending);
        let images = vec![
            make_sized_source_image("charlie", 32, 32),
            make_sized_source_image("alpha", 32, 32),
            make_sized_source_image("bravo", 32, 32),
        ];

        let atlas = try_pack(&images, 256, 256, &settings).expect("must fit");

        assert_eq!(atlas.placements[0].name, "charlie", "descending name sort must place 'charlie' first");
    }

    #[test]
    fn shelf_width_sort_ascending_first_placement_is_narrowest() {
        let settings = shelf_sort_settings(ShelfSortBy::Width, SortOrder::Ascending);
        let images = vec![
            make_sized_source_image("wide", 64, 32),
            make_sized_source_image("narrow", 16, 32),
            make_sized_source_image("medium", 32, 32),
        ];

        let atlas = try_pack(&images, 256, 256, &settings).expect("must fit");

        assert_eq!(atlas.placements[0].name, "narrow", "ascending width sort must place narrowest sprite first");
    }

    #[test]
    fn shelf_sort_still_produces_valid_placements() {
        for sort_by in [
            ShelfSortBy::Best,
            ShelfSortBy::Name,
            ShelfSortBy::Width,
            ShelfSortBy::Height,
            ShelfSortBy::Circumference,
        ] {
            for order in [SortOrder::Ascending, SortOrder::Descending] {
                let settings = shelf_sort_settings(sort_by, order);
                let images: Vec<SourceImage> = (0..5).map(|i| make_source_image(&format!("img{i}"), 32)).collect();

                let atlas = try_pack(&images, 256, 256, &settings).expect("sprites must fit for all sort options");

                for i in 0..atlas.placements.len() {
                    for j in (i + 1)..atlas.placements.len() {
                        let a = &atlas.placements[i];
                        let b = &atlas.placements[j];
                        assert!(!placements_overlap(a, b), "sort {:?}/{:?}: '{}' and '{}' overlap", sort_by, order, a.name, b.name);
                    }
                }
            }
        }
    }
}
