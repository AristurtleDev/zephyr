// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use serde::{Deserialize, Serialize};

use super::enums::PivotPreset;
use super::hitbox::HitboxShape;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PivotPoint {
    pub(crate) preset: PivotPreset,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Default for PivotPoint {
    fn default() -> Self {
        let (x, y) = PivotPreset::Center.normalized_coords();
        Self { preset: PivotPreset::Center, x, y }
    }
}

impl PivotPoint {
    pub(crate) fn apply_preset(&mut self, preset: PivotPreset) {
        self.preset = preset;
        if preset != PivotPreset::Custom {
            let (x, y) = preset.normalized_coords();
            self.x = x;
            self.y = y;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct SpriteProperties {
    pub(crate) pivot_enabled: bool,
    pub(crate) pivot: PivotPoint,
    pub(crate) hitbox_enabled: bool,
    pub(crate) hitbox: HitboxShape,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pivot_apply_preset() {
        let mut pivot = PivotPoint::default();
        pivot.apply_preset(PivotPreset::TopLeft);
        assert_eq!(pivot.x, 0.0);
        assert_eq!(pivot.y, 0.0);
    }

    #[test]
    fn pivot_custom_does_not_overwrite_coordinates() {
        let mut pivot = PivotPoint::default();
        pivot.x = 0.25;
        pivot.y = 0.75;
        pivot.apply_preset(PivotPreset::Custom);
        assert_eq!(pivot.x, 0.25, "Custom preset must leave x unchanged");
        assert_eq!(pivot.y, 0.75, "Custom preset must leave y unchanged");
    }
}
