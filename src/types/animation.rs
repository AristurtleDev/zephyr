// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

use serde::{Deserialize, Serialize};

use super::enums::PlaybackDirection;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct AnimationFrame {
    pub(crate) sprite_name: String,
    pub(crate) delay_ms: u32,
}

impl AnimationFrame {
    pub(crate) fn new(sprite_name: impl Into<String>) -> Self {
        Self {
            sprite_name: sprite_name.into(),
            delay_ms: 100,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Animation {
    pub(crate) name: String,
    pub(crate) frames: Vec<AnimationFrame>,
    pub(crate) direction: PlaybackDirection,

    #[serde(default = "default_loop_enabled")]
    pub(crate) loop_enabled: bool,
}

fn default_loop_enabled() -> bool {
    true
}

impl Animation {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            frames: Vec::new(),
            direction: PlaybackDirection::Forward,
            loop_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_frame_new_defaults_delay() {
        let frame = AnimationFrame::new("walk_0");
        assert_eq!(frame.sprite_name, "walk_0");
        assert_eq!(frame.delay_ms, 100);
    }

    #[test]
    fn animation_new_starts_empty() {
        let anim = Animation::new("run");
        assert_eq!(anim.name, "run");
        assert!(anim.frames.is_empty());
        assert_eq!(anim.direction, PlaybackDirection::Forward);
        assert!(anim.loop_enabled);
    }

    #[test]
    fn animation_new_defaults_to_looping_forward() {
        let anim = Animation::new("idle");
        assert!(anim.loop_enabled, "new animations loop by default");
        assert_eq!(anim.direction, PlaybackDirection::Forward);
    }
}
