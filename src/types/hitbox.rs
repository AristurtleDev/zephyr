// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use serde::{Deserialize, Serialize};

use super::enums::HitboxType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum HitboxShape {
    Rectangle { x: f32, y: f32, w: f32, h: f32 },
    Circle { cx: f32, cy: f32, radius: f32 },

    // Convex or concave polygon. Each element is `[x, y]` in sprite-local
    // pixel coordinates. The polygon is considered closed; no duplicate of
    // the first point is required.
    Polygon { points: Vec<[f32; 2]> },
}

impl Default for HitboxShape {
    fn default() -> Self {
        Self::Rectangle { x: 0.0, y: 0.0, w: 32.0, h: 32.0 }
    }
}

impl HitboxShape {
    pub(crate) fn hitbox_type(&self) -> HitboxType {
        match self {
            Self::Rectangle { .. } => HitboxType::Rectangle,
            Self::Circle { .. } => HitboxType::Circle,
            Self::Polygon { .. } => HitboxType::Polygon,
        }
    }

    pub(crate) fn default_for_type(hitbox_type: HitboxType, width: f32, height: f32) -> Self {
        match hitbox_type {
            HitboxType::Rectangle => Self::Rectangle { x: 0.0, y: 0.0, w: width, h: height },
            HitboxType::Circle => Self::Circle {
                cx: width / 2.0,
                cy: height / 2.0,
                radius: width.min(height) / 2.0,
            },
            HitboxType::Polygon => Self::Polygon { points: Vec::new() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hitbox_default_for_type() {
        let rect = HitboxShape::default_for_type(HitboxType::Rectangle, 64.0, 32.0);
        assert!(matches!(rect, HitboxShape::Rectangle { w, h, .. } if w == 64.0 && h == 32.0));
    }

    #[test]
    fn hitbox_type_discriminant_matches_shape() {
        assert_eq!(HitboxShape::Circle { cx: 0.0, cy: 0.0, radius: 1.0 }.hitbox_type(), HitboxType::Circle,);
        assert_eq!(HitboxShape::Polygon { points: vec![] }.hitbox_type(), HitboxType::Polygon,);
    }
}
