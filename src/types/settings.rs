// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use serde::{Deserialize, Serialize};

use super::enums::{AlphaProcessing, PackingAlgorithm, PixelFormat, ShelfSortBy, SortOrder, TextureFormat, TrimMode};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PackSettings {
    pub(crate) shape_padding: u32,
    pub(crate) border_padding: u32,
    pub(crate) extrude: u32,
    pub(crate) algorithm: PackingAlgorithm,
    pub(crate) force_square: bool,
    #[serde(default)]
    pub(crate) grid_alignment: u32,
    #[serde(default)]
    pub(crate) trim_mode: TrimMode,
    #[serde(default)]
    pub(crate) shelf_sort_by: ShelfSortBy,
    #[serde(default)]
    pub(crate) shelf_sort_order: SortOrder,

    // Texture settings
    pub(crate) texture_format: TextureFormat,
    pub(crate) pixel_format: PixelFormat,
    pub(crate) alpha_processing: AlphaProcessing,

    // Format-specific settings
    pub(crate) png_compression_level: u8,
    pub(crate) jpeg_quality: u8,
    pub(crate) compression_quality: u8,
    pub(crate) generate_mipmaps: bool,
}

impl Default for PackSettings {
    fn default() -> Self {
        Self {
            shape_padding: 2,
            border_padding: 2,
            extrude: 0,
            algorithm: PackingAlgorithm::default(),
            force_square: true,
            grid_alignment: 0,
            trim_mode: TrimMode::Off,
            shelf_sort_by: ShelfSortBy::default(),
            shelf_sort_order: SortOrder::default(),
            texture_format: TextureFormat::default(),
            pixel_format: PixelFormat::default(),
            alpha_processing: AlphaProcessing::default(),
            png_compression_level: 0,
            jpeg_quality: 80,
            compression_quality: 80,
            generate_mipmaps: false,
        }
    }
}

impl PackSettings {
    pub(crate) fn validate_formats(&mut self) {
        if !self.texture_format.supports_pixel_format(self.pixel_format) {
            self.pixel_format = self.texture_format.default_pixel_format();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_mode_round_trips() {
        let mut settings = PackSettings::default();
        settings.trim_mode = TrimMode::Trim;
        let json = serde_json::to_string(&settings).unwrap();
        let recovered: PackSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.trim_mode, TrimMode::Trim);
    }

    #[test]
    fn validate_formats_jpeg_rgba8_corrects_to_rgb8() {
        let mut settings = PackSettings::default();
        settings.texture_format = TextureFormat::Jpeg;
        settings.pixel_format = PixelFormat::RGBA8;
        settings.validate_formats();
        assert_eq!(
            settings.pixel_format,
            PixelFormat::RGB8,
            "JPEG does not support RGBA8; validate_formats must correct to RGB8",
        );
    }

    #[test]
    fn shelf_sort_fields_round_trip() {
        let mut settings = PackSettings::default();
        settings.shelf_sort_by = ShelfSortBy::Name;
        settings.shelf_sort_order = SortOrder::Ascending;
        let json = serde_json::to_string(&settings).unwrap();
        let recovered: PackSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.shelf_sort_by, ShelfSortBy::Name);
        assert_eq!(recovered.shelf_sort_order, SortOrder::Ascending);
    }

    #[test]
    fn validate_formats_png_rgb8_unchanged() {
        let mut settings = PackSettings::default();
        settings.texture_format = TextureFormat::Png;
        settings.pixel_format = PixelFormat::RGB8;
        settings.validate_formats();
        assert_eq!(
            settings.pixel_format,
            PixelFormat::RGB8,
            "PNG + RGB8 is a valid combination; validate_formats must not change it",
        );
    }
}
