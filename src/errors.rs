// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum LoadError {
    #[error("Failed to open image: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to decode image: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Invalid file name")]
    InvalidFileName,
}

#[derive(Error, Debug)]
pub(crate) enum ExportError {
    #[error("Failed to save image: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Failed to serialize JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to write file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid format combination: {0}")]
    InvalidFormatCombination(String),
}
