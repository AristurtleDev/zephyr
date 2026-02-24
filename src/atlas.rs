// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use crate::types::{AlphaProcessing, NodeId, Placement, SourceImage};
use image::{RgbaImage, imageops};
use std::collections::HashMap;

pub(crate) fn create_atlas_texture(placements: &[Placement], images: &[SourceImage], width: u32, height: u32, extrude: u32, alpha_processing: AlphaProcessing) -> RgbaImage {
    let mut atlas = RgbaImage::new(width, height);

    // Keyed by NodeId rather than name so that two images with the same filename
    // but different directory roots remain distinct entries.
    let image_map: HashMap<NodeId, &SourceImage> = images.iter().map(|img| (img.node_id, img)).collect();

    for placement in placements {
        if let Some(src_img) = image_map.get(&placement.node_id) {
            let processed_image = apply_alpha_processing(&src_img.data, alpha_processing);

            if let Err(e) = copy_image_to_atlas(&mut atlas, &processed_image, placement.x, placement.y, extrude) {
                eprintln!("Warning: Failed to copy image '{}': {}", placement.name, e);
            }
        }
    }

    atlas
}

fn apply_alpha_processing(source: &RgbaImage, mode: AlphaProcessing) -> RgbaImage {
    match mode {
        AlphaProcessing::None => source.clone(),
        AlphaProcessing::OptimizeCompression => clear_transparent_pixels(source),
        AlphaProcessing::Premultiplied => premultiply_alpha(source),
    }
}

fn clear_transparent_pixels(source: &RgbaImage) -> RgbaImage {
    let mut result = source.clone();

    for pixel in result.pixels_mut() {
        if pixel[3] == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        }
    }

    result
}

fn premultiply_alpha(source: &RgbaImage) -> RgbaImage {
    let mut result = source.clone();

    for pixel in result.pixels_mut() {
        let alpha = pixel[3] as f32 / 255.0;
        pixel[0] = (pixel[0] as f32 * alpha) as u8;
        pixel[1] = (pixel[1] as f32 * alpha) as u8;
        pixel[2] = (pixel[2] as f32 * alpha) as u8;
    }

    result
}

fn copy_image_to_atlas(atlas: &mut RgbaImage, source: &RgbaImage, offset_x: u32, offset_y: u32, extrude: u32) -> Result<(), String> {
    let (src_width, src_height) = source.dimensions();
    let (atlas_width, atlas_height) = atlas.dimensions();

    let total_width = src_width + extrude * 2;
    let total_height = src_height + extrude * 2;
    let start_x = offset_x.saturating_sub(extrude);
    let start_y = offset_y.saturating_sub(extrude);

    if start_x > atlas_width.saturating_sub(total_width) || start_y > atlas_height.saturating_sub(total_height) {
        return Err(format!(
            "Image at ({offset_x}, {offset_y}) with size {src_width}x{src_height} and extrude {extrude} exceeds atlas bounds {atlas_width}x{atlas_height}"
        ));
    }

    if extrude > 0 {
        let corner_pixel = source.get_pixel(0, 0);
        for dy in 0..extrude {
            for dx in 0..extrude {
                atlas.put_pixel(start_x + dx, start_y + dy, *corner_pixel);
            }
        }

        for x in 0..src_width {
            let edge_pixel = source.get_pixel(x, 0);
            for dy in 0..extrude {
                atlas.put_pixel(start_x + extrude + x, start_y + dy, *edge_pixel);
            }
        }

        let corner_pixel = source.get_pixel(src_width - 1, 0);
        for dy in 0..extrude {
            for dx in 0..extrude {
                atlas.put_pixel(start_x + extrude + src_width + dx, start_y + dy, *corner_pixel);
            }
        }

        for y in 0..src_height {
            let edge_pixel = source.get_pixel(0, y);
            for dx in 0..extrude {
                atlas.put_pixel(start_x + dx, start_y + extrude + y, *edge_pixel);
            }
        }

        for y in 0..src_height {
            let edge_pixel = source.get_pixel(src_width - 1, y);
            for dx in 0..extrude {
                atlas.put_pixel(start_x + extrude + src_width + dx, start_y + extrude + y, *edge_pixel);
            }
        }

        let corner_pixel = source.get_pixel(0, src_height - 1);
        for dy in 0..extrude {
            for dx in 0..extrude {
                atlas.put_pixel(start_x + dx, start_y + extrude + src_height + dy, *corner_pixel);
            }
        }

        for x in 0..src_width {
            let edge_pixel = source.get_pixel(x, src_height - 1);
            for dy in 0..extrude {
                atlas.put_pixel(start_x + extrude + x, start_y + extrude + src_height + dy, *edge_pixel);
            }
        }

        let corner_pixel = source.get_pixel(src_width - 1, src_height - 1);
        for dy in 0..extrude {
            for dx in 0..extrude {
                atlas.put_pixel(start_x + extrude + src_width + dx, start_y + extrude + src_height + dy, *corner_pixel);
            }
        }
    }

    for y in 0..src_height {
        for x in 0..src_width {
            let pixel = source.get_pixel(x, y);
            atlas.put_pixel(offset_x + x, offset_y + y, *pixel);
        }
    }

    Ok(())
}

/// Returns the tight bounding box `(x, y, w, h)` of all pixels with alpha > 0.
///
/// When every pixel is transparent the original full dimensions are returned
pub(crate) fn compute_trim_rect(img: &RgbaImage) -> (u32, u32, u32, u32) {
    let (width, height) = img.dimensions();
    if width == 0 || height == 0 {
        return (0, 0, width, height);
    }

    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut found_opaque = false;

    for y in 0..height {
        for x in 0..width {
            if img.get_pixel(x, y)[3] > 0 {
                if x < min_x {
                    min_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if x > max_x {
                    max_x = x;
                }
                if y > max_y {
                    max_y = y;
                }
                found_opaque = true;
            }
        }
    }

    if !found_opaque {
        return (0, 0, width, height);
    }

    (min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
}

/// Crops each image to its tight non-transparent bounding box.
///
/// Trim offset and original dimensions are stored on the returned `SourceImage`
/// so that packing and export can produce correct alignment metadata.
/// Fully-transparent images are returned unchanged to avoid zero-size sprites.
pub(crate) fn apply_trim(images: Vec<SourceImage>) -> Vec<SourceImage> {
    images
        .into_iter()
        .map(|img| {
            let (trim_x, trim_y, trim_w, trim_h) = compute_trim_rect(&img.data);
            let full_width = img.width;
            let full_height = img.height;

            if trim_x == 0 && trim_y == 0 && trim_w == full_width && trim_h == full_height {
                return SourceImage {
                    source_width: full_width,
                    source_height: full_height,
                    ..img
                };
            }

            let cropped = imageops::crop_imm(&img.data, trim_x, trim_y, trim_w, trim_h).to_image();
            SourceImage {
                data: cropped,
                width: trim_w,
                height: trim_h,
                source_width: full_width,
                source_height: full_height,
                trim_offset_x: trim_x,
                trim_offset_y: trim_y,
                name: img.name,
                path: img.path,
                node_id: img.node_id,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn make_rgba(width: u32, height: u32, fill: Rgba<u8>) -> RgbaImage {
        let mut img = RgbaImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = fill;
        }
        img
    }

    #[test]
    fn compute_trim_rect_fully_opaque_returns_full_rect() {
        let img = make_rgba(8, 8, Rgba([255, 0, 0, 255]));
        assert_eq!(compute_trim_rect(&img), (0, 0, 8, 8));
    }

    #[test]
    fn compute_trim_rect_fully_transparent_returns_full_rect() {
        let img = make_rgba(8, 8, Rgba([0, 0, 0, 0]));
        assert_eq!(compute_trim_rect(&img), (0, 0, 8, 8));
    }

    #[test]
    fn compute_trim_rect_transparent_border_returns_inner_rect() {
        // 8x8 image: only the centre 4x4 is opaque.
        let mut img = make_rgba(8, 8, Rgba([0, 0, 0, 0]));
        for y in 2..6u32 {
            for x in 2..6u32 {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        assert_eq!(compute_trim_rect(&img), (2, 2, 4, 4));
    }

    #[test]
    fn apply_trim_opaque_image_unchanged_dims() {
        let img = SourceImage {
            data: make_rgba(4, 4, Rgba([255, 0, 0, 255])),
            width: 4,
            height: 4,
            source_width: 4,
            source_height: 4,
            trim_offset_x: 0,
            trim_offset_y: 0,
            name: "s".to_string(),
            path: None,
            node_id: 0,
        };
        let result = apply_trim(vec![img]);
        let out = &result[0];
        assert_eq!(out.width, 4);
        assert_eq!(out.height, 4);
        assert_eq!(out.trim_offset_x, 0);
        assert_eq!(out.trim_offset_y, 0);
    }

    #[test]
    fn apply_trim_transparent_border_crops_correctly() {
        let mut data = make_rgba(8, 8, Rgba([0, 0, 0, 0]));
        for y in 2..6u32 {
            for x in 2..6u32 {
                data.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        let img = SourceImage {
            data,
            width: 8,
            height: 8,
            source_width: 8,
            source_height: 8,
            trim_offset_x: 0,
            trim_offset_y: 0,
            name: "s".to_string(),
            path: None,
            node_id: 0,
        };
        let result = apply_trim(vec![img]);
        let out = &result[0];
        assert_eq!(out.width, 4);
        assert_eq!(out.height, 4);
        assert_eq!(out.source_width, 8);
        assert_eq!(out.source_height, 8);
        assert_eq!(out.trim_offset_x, 2);
        assert_eq!(out.trim_offset_y, 2);
    }
}
