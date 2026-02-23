// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use crate::errors::LoadError;
use crate::types::SourceImage;
use image::ImageReader;
use std::path::Path;

pub(crate) fn load_image(path: &Path) -> Result<SourceImage, LoadError> {
    let img = ImageReader::open(path)?.decode()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let name = path.file_name().and_then(|n| n.to_str()).ok_or(LoadError::InvalidFileName)?.to_string();

    Ok(SourceImage {
        data: rgba,
        width,
        height,
        name,
        path: Some(path.to_path_buf()),
        source_width: width,
        source_height: height,
        trim_offset_x: 0,
        trim_offset_y: 0,
    })
}

pub(crate) fn load_images(paths: &[std::path::PathBuf]) -> Vec<SourceImage> {
    paths
        .iter()
        .filter_map(|p| match load_image(p) {
            Ok(img) => Some(img),
            Err(e) => {
                eprintln!("Failed to load {}: {}", p.display(), e);
                None
            }
        })
        .collect()
}
