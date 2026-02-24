// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

use crate::atlas::{apply_trim, create_atlas_texture};
use crate::loader::load_images;
use crate::packing::pack_images;
use crate::types::{PackSettings, PackingAlgorithm, PixelFormat, TextureFormat, TrimMode};

pub(crate) fn run(args: &[String]) -> ! {
    match args.first().map(|s| s.as_str()) {
        Some("export") => process::exit(run_export(&args[1..])),
        Some("--help") | Some("-h") | Some("help") => {
            print_help();
            process::exit(0);
        }
        Some("--version") | Some("-V") => {
            println!("zephyr {}", env!("CARGO_PKG_VERSION"));
            process::exit(0);
        }
        Some(cmd) => {
            eprintln!("error: unknown command '{}'", cmd);
            eprintln!("Run 'zephyr --help' for usage.");
            process::exit(1);
        }
        None => {
            print_help();
            process::exit(0);
        }
    }
}

struct ExportArgs {
    sources: Vec<PathBuf>,
    destination: String,
    settings: PackSettings,
}

fn parse_export_args(args: &[String]) -> Result<ExportArgs, String> {
    let mut positional: Vec<String> = Vec::new();
    let mut settings = PackSettings::default();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--shape-padding" => {
                i += 1;
                settings.shape_padding = parse_u32(args, i, "--shape-padding")?;
            }
            "--border-padding" => {
                i += 1;
                settings.border_padding = parse_u32(args, i, "--border-padding")?;
            }
            "--padding" => {
                i += 1;
                let v = parse_u32(args, i, "--padding")?;
                settings.shape_padding = v;
                settings.border_padding = v;
            }
            "--extrude" => {
                i += 1;
                settings.extrude = parse_u32(args, i, "--extrude")?;
            }
            "--algorithm" => {
                i += 1;
                let val = require_arg(args, i, "--algorithm")?;
                settings.algorithm = match val.to_lowercase().as_str() {
                    "shelf" => PackingAlgorithm::Shelf,
                    "maxrects" => PackingAlgorithm::MaxRects,
                    _ => {
                        return Err(format!(
                            "unknown algorithm '{}': expected 'shelf' or 'maxrects'",
                            val
                        ))
                    }
                };
            }
            "--trim" => {
                settings.trim_mode = TrimMode::Trim;
            }
            "--force-square" => {
                settings.force_square = true;
            }
            "--texture-format" => {
                i += 1;
                let val = require_arg(args, i, "--texture-format")?;
                settings.texture_format = match val.to_lowercase().as_str() {
                    "png" => TextureFormat::Png,
                    "jpg" | "jpeg" => TextureFormat::Jpeg,
                    "bmp" => TextureFormat::Bmp,
                    "tga" => TextureFormat::Tga,
                    "tiff" | "tif" => TextureFormat::Tiff,
                    "webp" => TextureFormat::WebP,
                    _ => {
                        return Err(format!(
                            "unknown texture format '{}': expected png, jpg, bmp, tga, tiff, or webp",
                            val
                        ))
                    }
                };
            }
            "--pixel-format" => {
                i += 1;
                let val = require_arg(args, i, "--pixel-format")?;
                settings.pixel_format = match val.to_lowercase().as_str() {
                    "rgb8" => PixelFormat::RGB8,
                    "rgba8" => PixelFormat::RGBA8,
                    "rgb32f" => PixelFormat::RGB32F,
                    "rgba32f" => PixelFormat::RGBA32F,
                    _ => {
                        return Err(format!(
                            "unknown pixel format '{}': expected rgb8, rgba8, rgb32f, or rgba32f",
                            val
                        ))
                    }
                };
            }
            "--jpeg-quality" => {
                i += 1;
                settings.jpeg_quality = parse_u8(args, i, "--jpeg-quality")?;
            }
            "--png-compression" => {
                i += 1;
                settings.png_compression_level = parse_u8(args, i, "--png-compression")?;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("unknown flag '{}'", flag));
            }
            _ => {
                positional.push(args[i].clone());
            }
        }
        i += 1;
    }

    // The last positional arg is the destination; everything before it is a source.
    if positional.len() < 2 {
        return Err(
            "missing required arguments: at least one <source> and a <destination> are required"
                .to_string(),
        );
    }

    let destination = positional.pop().unwrap();
    let sources = positional.into_iter().map(PathBuf::from).collect();

    settings.validate_formats();

    Ok(ExportArgs { sources, destination, settings })
}

fn require_arg<'a>(args: &'a [String], i: usize, flag: &str) -> Result<&'a str, String> {
    args.get(i)
        .filter(|s| !s.starts_with("--"))
        .map(|s| s.as_str())
        .ok_or_else(|| format!("{} requires a value", flag))
}

fn parse_u32(args: &[String], i: usize, flag: &str) -> Result<u32, String> {
    let s = require_arg(args, i, flag)?;
    s.parse::<u32>()
        .map_err(|_| format!("{} value '{}' is not a valid non-negative integer", flag, s))
}

fn parse_u8(args: &[String], i: usize, flag: &str) -> Result<u8, String> {
    let s = require_arg(args, i, flag)?;
    s.parse::<u8>()
        .map_err(|_| format!("{} value '{}' must be in range 0-255", flag, s))
}

fn run_export(args: &[String]) -> i32 {
    let parsed = match parse_export_args(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!("Usage: zephyr export <source> [<source> ...] <destination> [options]");
            eprintln!("Run 'zephyr --help' for full usage.");
            return 1;
        }
    };

    let mut image_paths: Vec<PathBuf> = Vec::new();
    for source in &parsed.sources {
        match collect_image_paths(source) {
            Ok(mut paths) => image_paths.append(&mut paths),
            Err(e) => {
                eprintln!("error: {}", e);
                return 1;
            }
        }
    }

    if image_paths.is_empty() {
        eprintln!("error: no images found in the provided sources");
        return 1;
    }

    let images = load_images(&image_paths);

    if images.is_empty() {
        eprintln!("error: no images could be loaded");
        return 1;
    }

    if images.len() < image_paths.len() {
        eprintln!(
            "warning: {}/{} images failed to load",
            image_paths.len() - images.len(),
            image_paths.len()
        );
    }

    let images_to_pack = if parsed.settings.trim_mode == TrimMode::Off {
        images
    } else {
        apply_trim(images)
    };

    let mut atlas = match pack_images(&images_to_pack, &parsed.settings) {
        Some(a) => a,
        None => {
            eprintln!("error: images do not fit within a 4096x4096 atlas");
            return 1;
        }
    };

    atlas.texture = create_atlas_texture(
        &atlas.placements,
        &images_to_pack,
        atlas.width,
        atlas.height,
        parsed.settings.extrude,
        parsed.settings.alpha_processing,
    );

    let dest = parsed.destination.as_str();
    let dest_path = Path::new(dest);

    // When the destination is (or looks like) a directory, default to "atlas" as
    // the file stem so the caller does not have to supply a name.
    let base_path =
        if dest_path.is_dir() || dest.ends_with('/') || dest.ends_with('\\') {
            dest_path.join("atlas").to_string_lossy().into_owned()
        } else {
            strip_known_extension(dest).to_string()
        };

    if let Some(parent) = Path::new(&base_path).parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!(
                    "error: failed to create output directory '{}': {}",
                    parent.display(),
                    e
                );
                return 1;
            }
        }
    }

    // A raw directory pack has no per-sprite metadata, so defaults are used.
    match crate::export::export_atlas(
        &atlas,
        &base_path,
        &parsed.settings,
        &HashMap::new(),
        &[],
    ) {
        Ok(_) => {
            let ext = texture_format_extension(parsed.settings.texture_format);
            println!("Exported {}.{} and {}.json", base_path, ext, base_path);
            0
        }
        Err(e) => {
            eprintln!("error: export failed: {}", e);
            1
        }
    }
}

fn collect_image_paths(source: &Path) -> Result<Vec<PathBuf>, String> {
    if !source.exists() {
        return Err(format!("source path '{}' does not exist", source.display()));
    }

    if source.is_file() {
        return Ok(vec![source.to_path_buf()]);
    }

    let mut paths = Vec::new();
    collect_images_recursive(source, &mut paths)
        .map_err(|e| format!("failed to read directory '{}': {}", source.display(), e))?;

    Ok(paths)
}

fn collect_images_recursive(dir: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_images_recursive(&path, paths)?;
        } else if let Some(ext) = path.extension() {
            if matches!(ext.to_string_lossy().to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "bmp") {
                paths.push(path);
            }
        }
    }
    Ok(())
}

// Strips a recognized image or data file extension so callers can pass a path
// like "./out/atlas.png" and still get the correct base path for both outputs.
fn strip_known_extension(path: &str) -> &str {
    const EXTENSIONS: &[&str] = &[
        ".png", ".jpg", ".jpeg", ".bmp", ".tga", ".tiff", ".tif", ".webp", ".json",
    ];
    for ext in EXTENSIONS {
        if let Some(base) = path.strip_suffix(ext) {
            return base;
        }
    }
    path
}

fn texture_format_extension(format: TextureFormat) -> &'static str {
    match format {
        TextureFormat::Png => "png",
        TextureFormat::Jpeg => "jpg",
        TextureFormat::Bmp => "bmp",
        TextureFormat::Tga => "tga",
        TextureFormat::Tiff => "tiff",
        TextureFormat::WebP => "webp",
    }
}

fn print_help() {
    println!(
        "Zephyr Texture Packer {}

Usage:
    zephyr [command]
    zephyr export <source> [<source> ...] <destination> [options]

Commands:
    export    Pack images into a texture atlas

Options:
    -h, --help       Print this help message
    -V, --version    Print version information

Export arguments:
    <source>        An image file (PNG, JPG, BMP) or a directory of images to pack.
                    Multiple sources may be given and are all packed into one atlas.
                    Directories are searched recursively.
    <destination>   Output directory or base path for the exported files (last argument).
                    When a directory is given, files are written as 'atlas.<ext>'
                    and 'atlas.json'. When a base path is given (e.g. ./out/sprites),
                    files are written as 'sprites.<ext>' and 'sprites.json'.

Export options:
    --shape-padding <int>       Padding around each sprite in pixels (default: 2)
    --border-padding <int>      Padding along the atlas border in pixels (default: 2)
    --padding <int>             Sets both shape and border padding
    --extrude <int>             Repeat the sprite edge outward N pixels (default: 0)
    --algorithm <name>          Packing algorithm: shelf, maxrects (default: shelf)
    --trim                      Remove transparent borders from sprites
    --force-square              Force square atlas dimensions
    --texture-format <fmt>      Output format: png, jpg, bmp, tga, tiff, webp (default: png)
    --pixel-format <fmt>        Pixel format: rgb8, rgba8, rgb32f, rgba32f (default: rgba8)
    --jpeg-quality <int>        JPEG quality 0-255 (default: 80)
    --png-compression <int>     PNG compression level 0-9 (default: 0)

Examples:
    zephyr export ./sprites ./output
    zephyr export hero.png enemy.png ./output/atlas
    zephyr export ./characters ./items ./output --algorithm maxrects --trim
    zephyr export ./sprites ./out --texture-format jpg --jpeg-quality 90

Homepage: {}",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_HOMEPAGE")
    );
}
