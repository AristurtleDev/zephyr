// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use std::collections::HashMap;

use crate::errors::ExportError;
use crate::types::{Animation, HitboxShape, PackSettings, PackedAtlas, PivotPoint, PixelFormat, SpriteProperties, TextureFormat};
use image::{DynamicImage, ImageBuffer, Rgb, RgbaImage};
use serde::Serialize;
use std::fs;
use std::path::Path;

// Increment this only when the exported JSON schema changes in a
// backwards-incompatible way. It is intentionally decoupled from the
// application version in Cargo.toml.
const EXPORT_FORMAT_VERSION: &str = "1.0";

#[derive(Serialize)]
struct AtlasMetadata {
    meta: Meta,
    frames: Vec<Frame>,
    animations: Vec<Animation>,
}

#[derive(Serialize)]
struct Meta {
    version: &'static str,
    app: &'static str,
    image: String,
    size: Size,
    format: String,
}

#[derive(Serialize)]
struct Size {
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct Frame {
    filename: String,
    frame: Rectangle,
    trimmed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_size: Option<Size>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<Offset>,
    pivot_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pivot: Option<PivotPoint>,
    hitbox_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    hitbox: Option<HitboxShape>,
}

#[derive(Serialize)]
struct Rectangle {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct Offset {
    x: u32,
    y: u32,
}

pub(crate) fn export_atlas(
    atlas: &PackedAtlas,
    output_path: &str,
    settings: &PackSettings,
    sprite_properties: &HashMap<String, SpriteProperties>,
    animations: &[Animation],
) -> Result<(), ExportError> {
    if !settings.texture_format.supports_pixel_format(settings.pixel_format) {
        return Err(ExportError::InvalidFormatCombination(format!(
            "{:?} does not support {:?}",
            settings.texture_format, settings.pixel_format
        )));
    }

    let extension = get_file_extension(settings.texture_format);
    let texture_path = format!("{}.{}", output_path, extension);

    let converted_image = convert_pixel_format(&atlas.texture, settings.pixel_format)?;

    export_texture(&converted_image, &texture_path, settings)?;
    export_metadata(atlas, output_path, &texture_path, settings, sprite_properties, animations)?;

    Ok(())
}

fn get_file_extension(format: TextureFormat) -> &'static str {
    match format {
        TextureFormat::Png => "png",
        TextureFormat::Jpeg => "jpg",
        TextureFormat::Bmp => "bmp",
        TextureFormat::Tga => "tga",
        TextureFormat::Tiff => "tiff",
        TextureFormat::WebP => "webp",
    }
}

fn convert_pixel_format(image: &RgbaImage, target: PixelFormat) -> Result<DynamicImage, ExportError> {
    match target {
        PixelFormat::RGB8 => {
            let (width, height) = image.dimensions();
            let mut rgb_image = ImageBuffer::new(width, height);

            for (x, y, pixel) in image.enumerate_pixels() {
                rgb_image.put_pixel(x, y, Rgb([pixel[0], pixel[1], pixel[2]]));
            }

            Ok(DynamicImage::ImageRgb8(rgb_image))
        }

        PixelFormat::RGBA8 => Ok(DynamicImage::ImageRgba8(image.clone())),

        PixelFormat::RGB32F => {
            let (width, height) = image.dimensions();
            let mut rgb_f32 = ImageBuffer::new(width, height);

            for (x, y, pixel) in image.enumerate_pixels() {
                let r = pixel[0] as f32 / 255.0;
                let g = pixel[1] as f32 / 255.0;
                let b = pixel[2] as f32 / 255.0;
                rgb_f32.put_pixel(x, y, Rgb([r, g, b]));
            }

            Ok(DynamicImage::ImageRgb32F(rgb_f32))
        }

        PixelFormat::RGBA32F => {
            let (width, height) = image.dimensions();
            let mut rgba_f32 = ImageBuffer::new(width, height);

            for (x, y, pixel) in image.enumerate_pixels() {
                let r = pixel[0] as f32 / 255.0;
                let g = pixel[1] as f32 / 255.0;
                let b = pixel[2] as f32 / 255.0;
                let a = pixel[3] as f32 / 255.0;
                rgba_f32.put_pixel(x, y, image::Rgba([r, g, b, a]));
            }

            Ok(DynamicImage::ImageRgba32F(rgba_f32))
        }
    }
}

fn export_texture(image: &DynamicImage, path: &str, settings: &PackSettings) -> Result<(), ExportError> {
    match settings.texture_format {
        TextureFormat::Png => {
            let file = fs::File::create(path)?;
            let encoder = image::codecs::png::PngEncoder::new_with_quality(file, image::codecs::png::CompressionType::Default, image::codecs::png::FilterType::Sub);

            image.write_with_encoder(encoder)?;
        }

        TextureFormat::Jpeg => {
            let mut file = fs::File::create(path)?;
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, settings.jpeg_quality);

            encoder.encode_image(image)?;
        }

        TextureFormat::Bmp => {
            image.save(path)?;
        }

        TextureFormat::Tga => {
            let file = fs::File::create(path)?;
            let encoder = image::codecs::tga::TgaEncoder::new(file);
            image.write_with_encoder(encoder)?;
        }

        TextureFormat::Tiff => {
            image.save(path)?;
        }

        TextureFormat::WebP => {
            // WebP lossless only (image crate doesn't support lossy)
            let file = fs::File::create(path)?;
            let encoder = image::codecs::webp::WebPEncoder::new_lossless(file);
            image.write_with_encoder(encoder)?;
        }
    }

    Ok(())
}

fn export_metadata(
    atlas: &PackedAtlas,
    output_base_path: &str,
    texture_filename: &str,
    settings: &PackSettings,
    sprite_properties: &HashMap<String, SpriteProperties>,
    animations: &[Animation],
) -> Result<(), ExportError> {
    let texture_name = Path::new(texture_filename).file_name().and_then(|n| n.to_str()).unwrap_or(texture_filename);

    let metadata = build_metadata(atlas, texture_name, settings, sprite_properties, animations);

    let json_path = format!("{}.json", output_base_path);
    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(&json_path, json)?;

    Ok(())
}

fn build_metadata(
    atlas: &PackedAtlas,
    texture_name: &str,
    settings: &PackSettings,
    sprite_properties: &HashMap<String, SpriteProperties>,
    animations: &[Animation],
) -> AtlasMetadata {
    AtlasMetadata {
        meta: Meta {
            version: EXPORT_FORMAT_VERSION,
            app: env!("CARGO_PKG_HOMEPAGE"),
            image: texture_name.to_string(),
            size: Size { w: atlas.width, h: atlas.height },
            format: pixel_format_to_string(settings.pixel_format),
        },
        frames: atlas
            .placements
            .iter()
            .map(|p| {
                let props = sprite_properties.get(&p.name).cloned().unwrap_or_default();
                let trimmed = p.trim_offset_x != 0 || p.trim_offset_y != 0 || p.source_width != p.width || p.source_height != p.height;
                let (original_size, offset) = if trimmed {
                    (
                        Some(Size { w: p.source_width, h: p.source_height }),
                        Some(Offset { x: p.trim_offset_x, y: p.trim_offset_y }),
                    )
                } else {
                    (None, None)
                };
                Frame {
                    filename: p.name.clone(),
                    frame: Rectangle { x: p.x, y: p.y, w: p.width, h: p.height },
                    trimmed,
                    original_size,
                    offset,
                    pivot_enabled: props.pivot_enabled,
                    pivot: if props.pivot_enabled { Some(props.pivot) } else { None },
                    hitbox_enabled: props.hitbox_enabled,
                    hitbox: if props.hitbox_enabled { Some(props.hitbox) } else { None },
                }
            })
            .collect(),
        animations: animations.to_vec(),
    }
}

fn pixel_format_to_string(format: PixelFormat) -> String {
    match format {
        PixelFormat::RGB8 => "RGB8",
        PixelFormat::RGBA8 => "RGBA8",
        PixelFormat::RGB32F => "RGB32F",
        PixelFormat::RGBA32F => "RGBA32F",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AnimationFrame, PivotPreset, Placement};
    use image::RgbaImage;

    #[test]
    fn test_pixel_format_conversion_rgb8() {
        let img = RgbaImage::new(2, 2);
        let result = convert_pixel_format(&img, PixelFormat::RGB8).unwrap();

        assert!(matches!(result, DynamicImage::ImageRgb8(_)));
    }

    #[test]
    fn test_format_validation() {
        let settings = PackSettings {
            texture_format: TextureFormat::Jpeg,
            pixel_format: PixelFormat::RGBA8,
            ..Default::default()
        };

        let atlas = PackedAtlas {
            texture: RgbaImage::new(64, 64),
            width: 64,
            height: 64,
            placements: vec![],
        };

        // Should fail, JPEG doesn't support RGBA8
        let result = export_atlas(&atlas, "/tmp/test", &settings, &HashMap::new(), &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_extension() {
        assert_eq!(get_file_extension(TextureFormat::Png), "png");
        assert_eq!(get_file_extension(TextureFormat::Jpeg), "jpg");
        assert_eq!(get_file_extension(TextureFormat::WebP), "webp");
    }

    #[test]
    fn build_metadata_includes_pivot_hitbox_and_animations() {
        let atlas = PackedAtlas {
            texture: RgbaImage::new(64, 64),
            width: 64,
            height: 64,
            placements: vec![Placement {
                name: "hero".to_string(),
                x: 0,
                y: 0,
                width: 32,
                height: 32,
                source_width: 32,
                source_height: 32,
                trim_offset_x: 0,
                trim_offset_y: 0,
                node_id: 0,
            }],
        };

        let mut sprite_properties = HashMap::new();
        sprite_properties.insert(
            "hero".to_string(),
            SpriteProperties {
                pivot_enabled: true,
                pivot: PivotPoint {
                    preset: PivotPreset::Center,
                    x: 0.5,
                    y: 0.5,
                },
                hitbox_enabled: true,
                hitbox: HitboxShape::Circle { cx: 16.0, cy: 16.0, radius: 12.0 },
            },
        );

        let mut anim = Animation::new("run");
        anim.frames.push(AnimationFrame::new("hero"));

        let metadata = build_metadata(&atlas, "atlas.png", &PackSettings::default(), &sprite_properties, &[anim]);

        let json = serde_json::to_string(&metadata).unwrap();

        assert!(json.contains("\"pivot_enabled\":true"), "pivot_enabled must be in JSON");
        assert!(json.contains("\"hitbox_enabled\":true"), "hitbox_enabled must be in JSON");
        assert!(json.contains("\"animations\""), "animations key must be in JSON");
        assert!(json.contains("\"run\""), "animation name must be in JSON");
        assert!(json.contains("\"hero\""), "frame sprite name must be in JSON");
        assert!(json.contains("Circle"), "hitbox shape type must be in JSON");
        assert!(json.contains("\"trimmed\":false"), "untrimmed frame must have trimmed=false");
        assert!(!json.contains("\"offset\""), "untrimmed frame must not emit offset");
    }

    #[test]
    fn build_metadata_emits_size_and_offset_only_when_trimmed() {
        let atlas = PackedAtlas {
            texture: RgbaImage::new(64, 64),
            width: 64,
            height: 64,
            placements: vec![
                Placement {
                    name: "trimmed".to_string(),
                    x: 0,
                    y: 0,
                    width: 20,
                    height: 20,
                    source_width: 32,
                    source_height: 32,
                    trim_offset_x: 4,
                    trim_offset_y: 6,
                    node_id: 0,
                },
                Placement {
                    name: "untrimmed".to_string(),
                    x: 20,
                    y: 0,
                    width: 32,
                    height: 32,
                    source_width: 32,
                    source_height: 32,
                    trim_offset_x: 0,
                    trim_offset_y: 0,
                    node_id: 0,
                },
            ],
        };

        let metadata = build_metadata(&atlas, "atlas.png", &PackSettings::default(), &HashMap::new(), &[]);
        let json = serde_json::to_string_pretty(&metadata).unwrap();

        assert!(json.contains("\"trimmed\": true"), "trimmed frame must be marked");
        assert!(json.contains("\"offset\""), "trimmed frame must emit offset");
        assert!(json.contains("\"original_size\""), "trimmed frame must emit original_size");
        assert!(json.contains("\"x\": 4"), "offset x must match trim_offset_x");
        assert!(json.contains("\"y\": 6"), "offset y must match trim_offset_y");
    }

    #[test]
    fn build_metadata_uses_defaults_for_sprite_with_no_properties() {
        let atlas = PackedAtlas {
            texture: RgbaImage::new(64, 64),
            width: 64,
            height: 64,
            placements: vec![Placement {
                name: "unknown".to_string(),
                x: 0,
                y: 0,
                width: 32,
                height: 32,
                source_width: 32,
                source_height: 32,
                trim_offset_x: 0,
                trim_offset_y: 0,
                node_id: 0,
            }],
        };

        // No entry in sprite_properties for "unknown" - defaults apply
        let metadata = build_metadata(&atlas, "atlas.png", &PackSettings::default(), &HashMap::new(), &[]);

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"pivot_enabled\":false"));
        assert!(json.contains("\"hitbox_enabled\":false"));
        assert!(!json.contains("\"pivot\""), "disabled pivot must not be emitted");
        assert!(!json.contains("\"hitbox\""), "disabled hitbox must not be emitted");
    }
}
