using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;

namespace ZephyrMonoGameExample;

class AnimatedSprite
{
    private readonly Texture2D _atlas;
    private readonly Dictionary<string, ZephyrFrame> _frames;
    private ZephyrAnimation _anim;

    private int _index;
    private double _elapsedMs;
    private ZephyrFrame _currentFrame;

    public bool FlipHorizontal { get; set; }

    // Logical bounding-box size of the current frame (original untrimmed dimensions).
    public int CurrentSourceW => _currentFrame.OriginalSize?.W ?? _currentFrame.Frame.W;
    public int CurrentSourceH => _currentFrame.OriginalSize?.H ?? _currentFrame.Frame.H;

    public AnimatedSprite(Texture2D atlas, Dictionary<string, ZephyrFrame> frames, ZephyrAnimation anim)
    {
        _atlas = atlas;
        _frames = frames;
        _anim = anim;
        _currentFrame = frames[anim.Frames[0].SpriteName];
    }

    public void SetAnimation(ZephyrAnimation anim)
    {
        if (_anim == anim)
        {
            return;
        }

        _anim = anim;
        _index = 0;
        _elapsedMs = 0;
        _currentFrame = _frames[_anim.Frames[0].SpriteName];
    }

    public void Update(GameTime gameTime)
    {
        _elapsedMs += gameTime.ElapsedGameTime.TotalMilliseconds;

        if (_elapsedMs >= _anim.Frames[_index].DelayMs)
        {
            _elapsedMs -= _anim.Frames[_index].DelayMs;
            int next = _index + 1;

            if (next < _anim.Frames.Count)
            {
                _index = next;
            }
            else if (_anim.LoopEnabled)
            {
                _index = 0;
            }
            // else hold on the last frame

            _currentFrame = _frames[_anim.Frames[_index].SpriteName];
        }
    }

    public void Draw(SpriteBatch? sb, Vector2 position, float scale = 1f)
    {
        ZephyrFrame frame = _currentFrame;
        SpriteEffects effects = FlipHorizontal ? SpriteEffects.FlipHorizontally : SpriteEffects.None;

        Rectangle src = new Rectangle(frame.Frame.X, frame.Frame.Y, frame.Frame.W, frame.Frame.H);

        int rawOffsetX = frame.Offset?.X ?? 0;
        int rawOffsetY = frame.Offset?.Y ?? 0;

        int sourceW = frame.OriginalSize?.W ?? frame.Frame.W;
        float offsetX = FlipHorizontal
            ? (sourceW - rawOffsetX - frame.Frame.W) * scale
            : rawOffsetX * scale;

        Rectangle dest = new Rectangle(
            (int)(position.X + offsetX),
            (int)(position.Y + rawOffsetY * scale),
            (int)(frame.Frame.W * scale),
            (int)(frame.Frame.H * scale)
        );

        sb?.Draw(_atlas, dest, src, Color.White, 0f, Vector2.Zero, effects, 0f);
    }
}
