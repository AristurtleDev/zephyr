using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace ZephyrMonoGameExample;

public class ZephyrAtlas
{
    [JsonPropertyName("meta")]
    public ZephyrMeta Meta { get; set; } = new();

    [JsonPropertyName("frames")]
    public List<ZephyrFrame> Frames { get; set; } = [];

    [JsonPropertyName("animations")]
    public List<ZephyrAnimation> Animations { get; set; } = [];

    public static ZephyrAtlas Load(Stream stream)
    {
        return JsonSerializer.Deserialize<ZephyrAtlas>(stream)
            ?? throw new InvalidOperationException("Could not parse atlas JSON.");
    }
}

public class ZephyrMeta
{
    [JsonPropertyName("image")]
    public string Image { get; set; } = string.Empty;

    [JsonPropertyName("size")]
    public ZephyrSize Size { get; set; } = new();
}

public class ZephyrRect
{
    [JsonPropertyName("x")]
    public int X { get; set; }

    [JsonPropertyName("y")]
    public int Y { get; set; }

    [JsonPropertyName("w")]
    public int W { get; set; }

    [JsonPropertyName("h")]
    public int H { get; set; }
}

public class ZephyrSize
{
    [JsonPropertyName("w")]
    public int W { get; set; }

    [JsonPropertyName("h")]
    public int H { get; set; }
}

// x/y draw offset; only present when the sprite was trimmed
public class ZephyrOffset
{
    [JsonPropertyName("x")]
    public int X { get; set; }

    [JsonPropertyName("y")]
    public int Y { get; set; }
}

public class ZephyrFrame
{
    [JsonPropertyName("filename")]
    public string Filename { get; set; } = string.Empty;

    [JsonPropertyName("frame")]
    public ZephyrRect Frame { get; set; } = new();

    [JsonPropertyName("trimmed")]
    public bool Trimmed { get; set; }

    // Original untrimmed sprite dimensions; only present when trimmed is true
    [JsonPropertyName("original_size")]
    public ZephyrSize? OriginalSize { get; set; }

    // Where to draw the trimmed frame within the original bounds; only present when trimmed is true
    [JsonPropertyName("offset")]
    public ZephyrOffset? Offset { get; set; }

    [JsonPropertyName("pivot_enabled")]
    public bool PivotEnabled { get; set; }

    [JsonPropertyName("pivot")]
    public ZephyrPivot? Pivot { get; set; }

    [JsonPropertyName("hitbox_enabled")]
    public bool HitboxEnabled { get; set; }

    [JsonPropertyName("hitbox")]
    public ZephyrHitbox? Hitbox { get; set; }
}

public class ZephyrPivot
{
    [JsonPropertyName("preset")]
    public string Preset { get; set; } = string.Empty;

    [JsonPropertyName("x")]
    public float X { get; set; }

    [JsonPropertyName("y")]
    public float Y { get; set; }
}

// The hitbox field is a tagged object: only the matching property will be non-null.
// e.g. { "Rectangle": { "x":0, "y":0, "w":18, "h":22 } }
public class ZephyrHitbox
{
    [JsonPropertyName("Rectangle")]
    public ZephyrRectF? Rectangle { get; set; }

    [JsonPropertyName("Circle")]
    public ZephyrCircle? Circle { get; set; }

    [JsonPropertyName("Polygon")]
    public ZephyrPolygon? Polygon { get; set; }
}

public class ZephyrRectF
{
    [JsonPropertyName("x")]
    public float X { get; set; }

    [JsonPropertyName("y")]
    public float Y { get; set; }

    [JsonPropertyName("w")]
    public float W { get; set; }

    [JsonPropertyName("h")]
    public float H { get; set; }
}

public class ZephyrCircle
{
    [JsonPropertyName("cx")]
    public float Cx { get; set; }

    [JsonPropertyName("cy")]
    public float Cy { get; set; }

    [JsonPropertyName("radius")]
    public float Radius { get; set; }
}

// Points are sprite-local pixel coordinates. The polygon is closed;
// no duplicate of the first point is stored.
public class ZephyrPolygon
{
    // Each element is [x, y]
    [JsonPropertyName("points")]
    public List<float[]> Points { get; set; } = [];
}

public class ZephyrAnimation
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("frames")]
    public List<ZephyrAnimFrame> Frames { get; set; } = [];

    [JsonPropertyName("direction")]
    public string Direction { get; set; } = "Forward";

    [JsonPropertyName("loop_enabled")]
    public bool LoopEnabled { get; set; }
}

public class ZephyrAnimFrame
{
    [JsonPropertyName("sprite_name")]
    public string SpriteName { get; set; } = string.Empty;

    [JsonPropertyName("delay_ms")]
    public int DelayMs { get; set; }
}
