# Zephyr Export Format

When you export from Zephyr, two files are written to your chosen output path:

- `<name>.png` (or `.jpg`, `.bmp`, etc.) — the packed texture atlas
- `<name>.json` — metadata describing every sprite, pivot, hitbox, and animation

This document describes the JSON format.

---

## Top-Level Structure

```json
{
  "meta": { ... },
  "frames": [ ... ],
  "animations": [ ... ]
}
```

| Field        | Type   | Description                           |
| ------------ | ------ | ------------------------------------- |
| `meta`       | object | Atlas-level information               |
| `frames`     | array  | One entry per packed sprite           |
| `animations` | array  | All animations defined in the project |

---

## `meta`

```json
"meta": {
  "version": "1.0",
  "app": "https://github.com/aristurtledev/zephyr",
  "image": "atlas.png",
  "size": { "w": 1024, "h": 1024 },
  "format": "RGBA8"
}
```

| Field     | Type    | Description                                                                                                       |
| --------- | ------- | ----------------------------------------------------------------------------------------------------------------- |
| `version` | string  | Export format version. Only increments on schema-breaking changes, independent of the Zephyr application version. |
| `app`     | string  | URL of the application that generated this file.                                                                  |
| `image`   | string  | Filename of the exported texture (just the name, no path)                                                         |
| `size.w`  | integer | Atlas width in pixels                                                                                             |
| `size.h`  | integer | Atlas height in pixels                                                                                            |
| `format`  | string  | Pixel format: `"RGB8"`, `"RGBA8"`, `"RGB32F"`, or `"RGBA32F"`                                                     |

---

## `frames`

Each element in the `frames` array describes one sprite.

```json
{
  "filename": "hero_idle_0",
  "frame": { "x": 0, "y": 0, "w": 40, "h": 56 },
  "trimmed": true,
  "original_size": { "w": 48, "h": 64 },
  "offset": { "x": 4, "y": 4 },
  "pivot_enabled": true,
  "pivot": { ... },
  "hitbox_enabled": true,
  "hitbox": { ... }
}
```

| Field            | Type    | Description                                                                                               |
| ---------------- | ------- | --------------------------------------------------------------------------------------------------------- |
| `filename`       | string  | Sprite name as it appears in the Zephyr tree                                                              |
| `frame`          | rect    | Position and size of the sprite on the atlas                                                              |
| `trimmed`        | boolean | Whether transparent border pixels were removed before packing                                             |
| `original_size`  | size    | Original image dimensions before trimming; only present when `trimmed` is `true`                          |
| `offset`         | point   | Top-left draw position of `frame` within the original image bounds; only present when `trimmed` is `true` |
| `pivot_enabled`  | boolean | Whether a pivot point is active for this sprite                                                           |
| `pivot`          | object  | Pivot point data (see below); only present when `pivot_enabled` is `true`                                 |
| `hitbox_enabled` | boolean | Whether a hitbox is active for this sprite                                                                |
| `hitbox`         | object  | Hitbox shape data (see below); only present when `hitbox_enabled` is `true`                               |

**Rect fields:** `x`, `y`, `w`, `h` — all integers, in pixels.
**Size and offset fields:** integers, in pixels.

### Trimming

When trim mode is enabled in atlas settings, transparent border pixels are stripped before packing. For trimmed sprites (`trimmed: true`), `original_size` gives the pre-trim canvas dimensions and `offset` gives the top-left pixel position at which to draw the trimmed frame within that canvas. Both fields are absent when `trimmed` is `false`.

---

## Pivot

The `pivot` object is only present in the JSON when `pivot_enabled` is `true`.

```json
"pivot": {
  "preset": "BottomCenter",
  "x": 0.5,
  "y": 1.0
}
```

| Field    | Type   | Description                                                            |
| -------- | ------ | ---------------------------------------------------------------------- |
| `preset` | string | Which preset was used (or `"Custom"` for a manually positioned pivot)  |
| `x`      | float  | Normalized horizontal position — `0.0` = left edge, `1.0` = right edge |
| `y`      | float  | Normalized vertical position — `0.0` = top edge, `1.0` = bottom edge   |

**Presets:** `"TopLeft"`, `"TopCenter"`, `"TopRight"`, `"MiddleLeft"`, `"Center"`, `"MiddleRight"`, `"BottomLeft"`, `"BottomCenter"`, `"BottomRight"`, `"Custom"`

To convert normalized pivot coordinates to pixels, multiply by the logical sprite dimensions: `pixel_x = x * original_size.w` and `pixel_y = y * original_size.h` for trimmed sprites, or `pixel_x = x * frame.w` and `pixel_y = y * frame.h` for untrimmed sprites.

---

## Hitbox

The `hitbox` object is only present in the JSON when `hitbox_enabled` is `true`.

All hitbox coordinates are in sprite-local pixel space, measured from the top-left of the **source image** (before trimming).

### Rectangle

```json
"hitbox": {
  "Rectangle": { "x": 4.0, "y": 8.0, "w": 40.0, "h": 48.0 }
}
```

| Field    | Description                      |
| -------- | -------------------------------- |
| `x`, `y` | Top-left corner of the rectangle |
| `w`, `h` | Width and height                 |

### Circle

```json
"hitbox": {
  "Circle": { "cx": 24.0, "cy": 32.0, "radius": 20.0 }
}
```

| Field      | Description          |
| ---------- | -------------------- |
| `cx`, `cy` | Center of the circle |
| `radius`   | Radius               |

### Polygon

```json
"hitbox": {
  "Polygon": {
    "points": [
      [0.0, 64.0],
      [24.0, 0.0],
      [48.0, 64.0]
    ]
  }
}
```

| Field    | Description                                                                                                        |
| -------- | ------------------------------------------------------------------------------------------------------------------ |
| `points` | Array of `[x, y]` vertices in order. The polygon is implicitly closed — no duplicate of the first point is needed. |

---

## `animations`

Each animation in the project is exported in full.

```json
{
  "name": "idle",
  "direction": "Forward",
  "loop_enabled": true,
  "frames": [
    { "sprite_name": "hero_idle_0", "delay_ms": 100 },
    { "sprite_name": "hero_idle_1", "delay_ms": 100 },
    { "sprite_name": "hero_idle_2", "delay_ms": 150 }
  ]
}
```

| Field                  | Type    | Description                                                                  |
| ---------------------- | ------- | ---------------------------------------------------------------------------- |
| `name`                 | string  | Animation name                                                               |
| `direction`            | string  | Playback direction (see below)                                               |
| `loop_enabled`         | boolean | Whether the animation loops                                                  |
| `frames`               | array   | Ordered list of frames                                                       |
| `frames[].sprite_name` | string  | Name of the sprite for this frame — matches `filename` in the `frames` array |
| `frames[].delay_ms`    | integer | How long this frame is displayed, in milliseconds                            |

**Directions:**

| Value        | Behavior                                |
| ------------ | --------------------------------------- |
| `"Forward"`  | Plays from the first frame to the last  |
| `"Reverse"`  | Plays from the last frame to the first  |
| `"PingPong"` | Alternates between forward and backward |

---

## Full Example

```json
{
  "meta": {
    "version": "1.0",
    "image": "atlas.png",
    "size": { "w": 256, "h": 256 },
    "format": "RGBA8"
  },
  "frames": [
    {
      "filename": "hero_idle_0",
      "frame": { "x": 0, "y": 0, "w": 40, "h": 56 },
      "trimmed": true,
      "original_size": { "w": 48, "h": 64 },
      "offset": { "x": 4, "y": 4 },
      "pivot_enabled": true,
      "pivot": { "preset": "BottomCenter", "x": 0.5, "y": 1.0 },
      "hitbox_enabled": true,
      "hitbox": {
        "Rectangle": { "x": 8.0, "y": 16.0, "w": 32.0, "h": 48.0 }
      }
    },
    {
      "filename": "background_tile",
      "frame": { "x": 40, "y": 0, "w": 16, "h": 16 },
      "trimmed": false,
      "pivot_enabled": false,
      "hitbox_enabled": false
    }
  ],
  "animations": [
    {
      "name": "idle",
      "direction": "Forward",
      "loop_enabled": true,
      "frames": [
        { "sprite_name": "hero_idle_0", "delay_ms": 120 },
        { "sprite_name": "hero_idle_1", "delay_ms": 120 },
        { "sprite_name": "hero_idle_2", "delay_ms": 120 }
      ]
    }
  ]
}
```
