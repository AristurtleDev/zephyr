using System;
using System.Collections.Generic;
using System.IO;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using Microsoft.Xna.Framework.Input;

namespace ZephyrMonoGameExample;

public class Game1 : Game
{
    private readonly GraphicsDeviceManager _graphics;
    private SpriteBatch? _spriteBatch;
    private AnimatedSprite? _player;
    private Dictionary<string, ZephyrAnimation> _animations = new();

    private Vector2 _position;
    private float _velocityY;
    private bool _isGrounded;
    private bool _isFacingLeft;

    private float _groundY;

    private const float Scale = 4.0f;
    private const float MoveSpeed = 300.0f;
    private const float JumpVelocity = -500.0f;
    private const float Gravity = 1200.0f;

    public Game1()
    {
        _graphics = new GraphicsDeviceManager(this);
        Content.RootDirectory = "Content";
        IsMouseVisible = true;
    }

    protected override void LoadContent()
    {
        _spriteBatch = new SpriteBatch(GraphicsDevice);

        ZephyrAtlas atlasData;
        using (Stream jsonStream = TitleContainer.OpenStream("Content/foxy.json"))
        {
            atlasData = ZephyrAtlas.Load(jsonStream);
        }

        Texture2D atlas;
        using (Stream pngStream = TitleContainer.OpenStream($"Content/{atlasData.Meta.Image}"))
        {
            atlas = Texture2D.FromStream(GraphicsDevice, pngStream);
        }

        Dictionary<string, ZephyrFrame> frameMap = new();
        foreach (ZephyrFrame frame in atlasData.Frames)
        {
            frameMap[frame.Filename] = frame;
        }

        foreach (ZephyrAnimation anim in atlasData.Animations)
        {
            _animations[anim.Name] = anim;
        }

        _player = new AnimatedSprite(atlas, frameMap, _animations["idle"]);

        // Stand the player on a ground line 75% down the viewport.
        // CurrentSourceW/H reflects the logical bounding box of the current frame.
        _groundY = GraphicsDevice.Viewport.Height * 0.75f;
        _position = new Vector2(
            GraphicsDevice.Viewport.Width * 0.5f - _player.CurrentSourceW * 0.5f * Scale,
            _groundY - _player.CurrentSourceH * Scale
        );

        _isGrounded = true;
    }

    protected override void Update(GameTime gameTime)
    {
        KeyboardState kb = Keyboard.GetState();

        if (kb.IsKeyDown(Keys.Escape))
            Exit();

        float dt = (float)gameTime.ElapsedGameTime.TotalSeconds;

        // Horizontal movement
        if (kb.IsKeyDown(Keys.Left))
        {
            _position.X -= MoveSpeed * dt;
            _isFacingLeft = true;
        }
        else if (kb.IsKeyDown(Keys.Right))
        {
            _position.X += MoveSpeed * dt;
            _isFacingLeft = false;
        }

        // Jump — only from the ground
        if (kb.IsKeyDown(Keys.Space) && _isGrounded)
        {
            _velocityY = JumpVelocity;
            _isGrounded = false;
        }

        if (_player != null)
        {
            // Vertical physics
            if (!_isGrounded)
            {
                _velocityY += Gravity * dt;
                _position.Y += _velocityY * dt;

                float groundedY = _groundY - _player.CurrentSourceH * Scale;
                if (_position.Y >= groundedY)
                {
                    _position.Y = groundedY;
                    _velocityY = 0;
                    _isGrounded = true;
                }
            }

            // Keep the player within the viewport horizontally
            float maxX = GraphicsDevice.Viewport.Width - _player.CurrentSourceW * Scale;
            _position.X = Math.Clamp(_position.X, 0f, maxX);

            string animName;
            if (!_isGrounded)
            {
                animName = "jump";
            }
            else if (kb.IsKeyDown(Keys.Down))
            {
                animName = "crouch";
            }
            else if (kb.IsKeyDown(Keys.Up))
            {
                animName = "look-up";
            }
            else if (kb.IsKeyDown(Keys.Left) || kb.IsKeyDown(Keys.Right))
            {
                animName = "run";
            }
            else
            {
                animName = "idle";
            }

            if (_animations.TryGetValue(animName, out ZephyrAnimation? anim))
            {
                _player.SetAnimation(anim);
            }

            _player.FlipHorizontal = _isFacingLeft;
            _player.Update(gameTime);
        }

        base.Update(gameTime);
    }

    protected override void Draw(GameTime gameTime)
    {
        GraphicsDevice.Clear(Color.CornflowerBlue);

        _spriteBatch?.Begin(samplerState: SamplerState.PointClamp);
        _player?.Draw(_spriteBatch, _position, Scale);
        _spriteBatch?.End();

        base.Draw(gameTime);
    }
}
