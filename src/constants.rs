use bevy::prelude::Vec2;

pub const PLAYER_SIZE: Vec2 = Vec2::new(96.0, 96.0);
pub const GROUND_SIZE: Vec2 = Vec2::new(2200.0, 120.0);
pub const PLAYER_SPEED: f32 = 420.0;
pub const JUMP_SPEED: f32 = 720.0;
pub const GRAVITY: f32 = 1800.0;
pub const GROUND_Y: f32 = -110.0;
pub const FLOOR_WIDTH: f32 = 900.0;

pub const DASH_SPEED: f32 = 1400.0;
pub const DASH_DURATION: f32 = 0.18;
pub const DOUBLE_TAP_TIMEOUT: f32 = 0.22;
pub const SMASHDOWN_SPEED: f32 = -2000.0;

pub const GLITCH_DURATION: f32 = 0.15;
pub const GLITCH_COOLDOWN: f32 = 1.0;
pub const GLITCH_DISTANCE: f32 = 180.0;
pub const WALL_SIZE: Vec2 = Vec2::new(40.0, 300.0);
