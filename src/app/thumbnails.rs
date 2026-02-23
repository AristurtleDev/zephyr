// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use crate::types::NodeId;
use egui::TextureHandle;
use image::RgbaImage;
use std::collections::HashMap;

const THUMBNAIL_MAX_PX: u32 = 32;

pub(crate) struct ThumbnailCache {
    cache: HashMap<NodeId, TextureHandle>,
}

impl ThumbnailCache {
    pub(crate) fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    pub(crate) fn get_or_insert(&mut self, id: NodeId, image: &RgbaImage, ctx: &egui::Context) -> &TextureHandle {
        self.cache.entry(id).or_insert_with(|| {
            let thumb = make_thumbnail(image, THUMBNAIL_MAX_PX);
            let size = [thumb.width() as usize, thumb.height() as usize];
            let pixels = thumb.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            let texture_options = egui::TextureOptions {
                magnification: egui::TextureFilter::Linear,
                minification: egui::TextureFilter::Linear,
                ..Default::default()
            };
            ctx.load_texture(format!("thumb_{}", id), color_image, texture_options)
        })
    }

    pub(crate) fn remove(&mut self, id: NodeId) -> Option<TextureHandle> {
        self.cache.remove(&id)
    }

    pub(crate) fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for ThumbnailCache {
    fn default() -> Self {
        Self::new()
    }
}

fn make_thumbnail(image: &RgbaImage, max_px: u32) -> RgbaImage {
    let w = image.width();
    let h = image.height();

    if w == 0 || h == 0 || (w <= max_px && h <= max_px) {
        return image.clone();
    }

    let scale = max_px as f32 / w.max(h) as f32;
    let new_w = ((w as f32 * scale).round() as u32).max(1);
    let new_h = ((h as f32 * scale).round() as u32).max(1);
    image::imageops::resize(image, new_w, new_h, image::imageops::FilterType::Triangle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_image_returned_unchanged() {
        let img = RgbaImage::new(16, 16);
        let thumb = make_thumbnail(&img, 32);
        assert_eq!(thumb.width(), 16);
        assert_eq!(thumb.height(), 16);
    }

    #[test]
    fn large_square_scaled_to_max() {
        let img = RgbaImage::new(128, 128);
        let thumb = make_thumbnail(&img, 32);
        assert_eq!(thumb.width(), 32);
        assert_eq!(thumb.height(), 32);
    }

    #[test]
    fn wide_image_fits_within_max() {
        let img = RgbaImage::new(128, 32);
        let thumb = make_thumbnail(&img, 32);
        assert!(thumb.width() <= 32);
        assert!(thumb.height() <= 32);
    }

    #[test]
    fn tall_image_fits_within_max() {
        let img = RgbaImage::new(32, 128);
        let thumb = make_thumbnail(&img, 32);
        assert!(thumb.width() <= 32);
        assert!(thumb.height() <= 32);
    }

    #[test]
    fn zero_dimension_image_returned_unchanged() {
        let img = RgbaImage::new(0, 0);
        let thumb = make_thumbnail(&img, 32);
        assert_eq!(thumb.width(), 0);
        assert_eq!(thumb.height(), 0);
    }
}
