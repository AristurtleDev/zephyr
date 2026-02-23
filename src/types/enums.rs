// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub(crate) enum PackingAlgorithm {
    #[default]
    Shelf,
    MaxRects,
}

impl PackingAlgorithm {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Shelf => "Shelf",
            Self::MaxRects => "MaxRects",
        }
    }

    pub(crate) const fn description(&self) -> &'static str {
        match self {
            Self::Shelf => "Fast, simple - good for sprites with similar heights",
            Self::MaxRects => "Best space efficiency - ideal for varied sprite sizes",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub(crate) enum TextureFormat {
    #[default]
    Png,
    Jpeg,
    Bmp,
    Tga,
    Tiff,
    WebP,
}

impl TextureFormat {
    pub(crate) const fn name(&self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Bmp => "BMP",
            Self::Tga => "TGA",
            Self::Tiff => "TIFF",
            Self::WebP => "WebP",
        }
    }

    pub(crate) const fn has_quality_setting(&self) -> bool {
        matches!(self, Self::Jpeg)
    }

    pub(crate) const fn has_compression_setting(&self) -> bool {
        matches!(self, Self::Png)
    }

    pub(crate) const fn valid_pixel_formats(&self) -> &'static [PixelFormat] {
        use PixelFormat::*;
        match self {
            Self::Png => &[RGB8, RGBA8, RGB32F, RGBA32F],
            Self::Jpeg => &[RGB8],
            Self::Bmp => &[RGB8, RGBA8],
            Self::Tga => &[RGB8, RGBA8],
            Self::Tiff => &[RGB8, RGBA8, RGB32F, RGBA32F],
            Self::WebP => &[RGB8, RGBA8],
        }
    }

    pub(crate) const fn supports_pixel_format(&self, format: PixelFormat) -> bool {
        let valid = self.valid_pixel_formats();
        let mut i = 0;
        while i < valid.len() {
            if (valid[i] as u8) == (format as u8) {
                return true;
            }
            i += 1;
        }
        false
    }

    pub(crate) const fn default_pixel_format(&self) -> PixelFormat {
        match self {
            Self::Jpeg => PixelFormat::RGB8,
            _ => PixelFormat::RGBA8,
        }
    }

    pub(crate) const fn description(&self) -> Option<&'static str> {
        match self {
            Self::WebP => Some("Lossless only (for lossy, use external tools)"),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub(crate) enum PixelFormat {
    RGB8,
    #[default]
    RGBA8,
    RGB32F,
    RGBA32F,
}

impl PixelFormat {
    pub(crate) const fn name(&self) -> &'static str {
        match self {
            Self::RGB8 => "RGB8",
            Self::RGBA8 => "RGBA8",
            Self::RGB32F => "RGB32F",
            Self::RGBA32F => "RGBA32F",
        }
    }

    pub(crate) const fn description(&self) -> &'static str {
        match self {
            Self::RGB8 => "RGB color, 8-bit per channel",
            Self::RGBA8 => "RGBA color with alpha, 8-bit per channel",
            Self::RGB32F => "RGB color, 32-bit float per channel",
            Self::RGBA32F => "RGBA color with alpha, 32-bit float per channel",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub(crate) enum AlphaProcessing {
    None,
    #[default]
    OptimizeCompression,
    Premultiplied,
}

impl AlphaProcessing {
    pub(crate) const fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::OptimizeCompression => "Optimize Compression",
            Self::Premultiplied => "Premultiplied",
        }
    }

    pub(crate) const fn description(&self) -> &'static str {
        match self {
            Self::None => "Keep original pixel data unchanged",
            Self::OptimizeCompression => "Zero out invisible pixels for smaller file sizes",
            Self::Premultiplied => "Multiply RGB values by alpha for premultiplied blending",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub(crate) enum PivotPreset {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    #[default]
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Custom,
}

impl PivotPreset {
    pub(crate) const fn normalized_coords(self) -> (f32, f32) {
        match self {
            Self::TopLeft => (0.0, 0.0),
            Self::TopCenter => (0.5, 0.0),
            Self::TopRight => (1.0, 0.0),
            Self::MiddleLeft => (0.0, 0.5),
            Self::Center => (0.5, 0.5),
            Self::MiddleRight => (1.0, 0.5),
            Self::BottomLeft => (0.0, 1.0),
            Self::BottomCenter => (0.5, 1.0),
            Self::BottomRight => (1.0, 1.0),
            Self::Custom => (0.5, 0.5),
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::TopLeft => "Top Left",
            Self::TopCenter => "Top Center",
            Self::TopRight => "Top Right",
            Self::MiddleLeft => "Middle Left",
            Self::Center => "Center",
            Self::MiddleRight => "Middle Right",
            Self::BottomLeft => "Bottom Left",
            Self::BottomCenter => "Bottom Center",
            Self::BottomRight => "Bottom Right",
            Self::Custom => "Custom",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub(crate) enum HitboxType {
    #[default]
    Rectangle,
    Circle,
    Polygon,
}

impl HitboxType {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Rectangle => "Rectangle",
            Self::Circle => "Circle",
            Self::Polygon => "Polygon",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub(crate) enum PlaybackDirection {
    #[default]
    Forward,
    Reverse,
    PingPong,
}

impl PlaybackDirection {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Forward => "Forward",
            Self::Reverse => "Reverse",
            Self::PingPong => "Ping-Pong",
        }
    }

    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::Forward => "Plays from the first frame to the last",
            Self::Reverse => "Plays from the last frame to the first",
            Self::PingPong => "Alternates between forward and backward playback",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub(crate) enum TrimMode {
    #[default]
    Off,
    Trim,
}

impl TrimMode {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Off => "None",
            Self::Trim => "Trim",
        }
    }

    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::Off => "Keep sprites at their original size. No transparent pixels are removed.",
            Self::Trim => "Remove the transparent border around each sprite. The original frame size and offset are stored in the export so renderers can restore alignment.",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub(crate) enum ShelfSortBy {
    #[default]
    Best,
    Directory,
    Name,
    Width,
    Height,
    Circumference,
}

impl ShelfSortBy {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Best => "Best",
            Self::Directory => "Directory",
            Self::Name => "Name",
            Self::Width => "Width",
            Self::Height => "Height",
            Self::Circumference => "Circumference",
        }
    }

    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::Best => "Sorts by the most efficient packing order for shelf layout",
            Self::Directory => "Sorts by the sprite's source directory path",
            Self::Name => "Sorts by sprite name",
            Self::Width => "Sorts by sprite width",
            Self::Height => "Sorts by sprite height",
            Self::Circumference => "Sorts by sprite perimeter (width + height)",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub(crate) enum SortOrder {
    Ascending,
    #[default]
    Descending,
}

impl SortOrder {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Ascending => "Ascending",
            Self::Descending => "Descending",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_validation_png() {
        assert!(TextureFormat::Png.supports_pixel_format(PixelFormat::RGBA8));
        assert!(TextureFormat::Png.supports_pixel_format(PixelFormat::RGB8));
    }

    #[test]
    fn test_format_validation_jpeg() {
        assert!(TextureFormat::Jpeg.supports_pixel_format(PixelFormat::RGB8));
        assert!(!TextureFormat::Jpeg.supports_pixel_format(PixelFormat::RGBA8));
    }

    #[test]
    fn test_default_pixel_formats() {
        assert_eq!(TextureFormat::Jpeg.default_pixel_format(), PixelFormat::RGB8);
        assert_eq!(TextureFormat::Png.default_pixel_format(), PixelFormat::RGBA8);
    }

    #[test]
    fn playback_direction_names_are_distinct() {
        assert_ne!(PlaybackDirection::Forward.name(), PlaybackDirection::Reverse.name());
        assert_ne!(PlaybackDirection::Forward.name(), PlaybackDirection::PingPong.name());
        assert_ne!(PlaybackDirection::Reverse.name(), PlaybackDirection::PingPong.name());
    }

    #[test]
    fn playback_direction_default_is_forward() {
        assert_eq!(PlaybackDirection::default(), PlaybackDirection::Forward);
    }

    #[test]
    fn trim_mode_default_is_off() {
        assert_eq!(TrimMode::default(), TrimMode::Off);
    }

    #[test]
    fn trim_mode_names_are_distinct() {
        assert_ne!(TrimMode::Off.name(), TrimMode::Trim.name());
    }

    #[test]
    fn shelf_sort_by_default_is_best() {
        assert_eq!(ShelfSortBy::default(), ShelfSortBy::Best);
    }

    #[test]
    fn shelf_sort_by_names_are_distinct() {
        let variants = [
            ShelfSortBy::Best,
            ShelfSortBy::Directory,
            ShelfSortBy::Name,
            ShelfSortBy::Width,
            ShelfSortBy::Height,
            ShelfSortBy::Circumference,
        ];
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                assert_ne!(variants[i].name(), variants[j].name());
            }
        }
    }

    #[test]
    fn sort_order_default_is_descending() {
        assert_eq!(SortOrder::default(), SortOrder::Descending);
    }

    #[test]
    fn sort_order_names_are_distinct() {
        assert_ne!(SortOrder::Ascending.name(), SortOrder::Descending.name());
    }
}
