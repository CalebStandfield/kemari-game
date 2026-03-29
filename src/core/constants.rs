pub const WINDOW_TITLE: &str = "Kemari MVP";
pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;

pub const COURT_WIDTH: f32 = 18.0;
pub const COURT_DEPTH: f32 = 12.0;
pub const COURT_Y: f32 = 0.0;

pub const PLAYER_WIDTH: f32 = 0.8;
pub const PLAYER_HEIGHT: f32 = 1.6;
pub const PLAYER_DEPTH: f32 = 0.8;
pub const PLAYER_COLLIDER_RADIUS: f32 = 0.45;
pub const PLAYER_Y: f32 = PLAYER_HEIGHT * 0.5;
pub const PLAYER_SPEED: f32 = 8.0;
pub const PLAYER_KICK_RANGE: f32 = 0.6;
pub const PLAYER_KICK_SPEED: f32 = 11.5;
pub const PLAYER_KICK_LIFT: f32 = 4.5;

pub const BALL_RADIUS: f32 = 0.35;
pub const BALL_START_X: f32 = 0.0;
pub const BALL_START_Y: f32 = BALL_RADIUS;
pub const BALL_START_Z: f32 = 0.0;
pub const BALL_GRAVITY: f32 = 24.0;
pub const BALL_GROUND_BOUNCE: f32 = 0.45;
pub const BALL_WALL_BOUNCE: f32 = 0.70;
pub const BALL_AIR_DRAG: f32 = 0.25;
pub const BALL_GROUND_FRICTION: f32 = 5.5;
pub const BALL_PLAYER_RESTITUTION: f32 = 0.85;
pub const BALL_PLAYER_PUSH_SPEED: f32 = 6.0;
pub const BALL_MIN_BOUNCE_SPEED: f32 = 0.9;
pub const BALL_MAX_HORIZONTAL_SPEED: f32 = 14.0;
