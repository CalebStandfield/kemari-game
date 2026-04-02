pub const WINDOW_TITLE: &str = "Kemari MVP";
pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;

pub const COURT_WIDTH: f32 = 18.0;
pub const COURT_DEPTH: f32 = 18.0;
pub const COURT_Y: f32 = 0.0;

pub const COURTYARD_SCENE_PATH: &str = "models/traditional_chinese_siheyuan_courtyard.glb#Scene0";
pub const COURTYARD_SCALE: f32 = 4.0;
pub const COURTYARD_OFFSET_X: f32 = -1.90;
pub const COURTYARD_OFFSET_Z: f32 = -1.0;
pub const COURTYARD_OFFSET_Y: f32 = 0.0;

pub const COURT_CORNER_TREE_PADDING: f32 = 0.25;
pub const COURT_CORNER_TREE_SCALE: f32 = 15.0;
pub const COURT_CORNER_TREE_SCENE_PATH: &str = "models/sakura_tree_01_-_low_poly_model.glb#Scene0";

pub const PLAYER_WIDTH: f32 = 0.8;
pub const PLAYER_HEIGHT: f32 = 1.6;
pub const PLAYER_DEPTH: f32 = 0.8;
pub const PLAYER_COLLIDER_RADIUS: f32 = 0.45;
pub const PLAYER_Y: f32 = PLAYER_HEIGHT * 0.5;
pub const PLAYER_SPEED: f32 = 8.0;

pub const TOUCH_COOLDOWN_KICK: f32 = 0.28;
pub const TOUCH_COOLDOWN_HEAD: f32 = 0.45;
pub const TOUCH_COOLDOWN_JUGGLE: f32 = 0.14;

pub const TOUCH_RADIUS_KICK: f32 = 1.45;
pub const TOUCH_RADIUS_HEAD: f32 = 1.05;
pub const TOUCH_RADIUS_JUGGLE: f32 = 1.20;

pub const TOUCH_HEIGHT_KICK_MIN: f32 = 0.25;
pub const TOUCH_HEIGHT_KICK_MAX: f32 = 1.10;
pub const TOUCH_HEIGHT_HEAD_MIN: f32 = 1.20;
pub const TOUCH_HEIGHT_HEAD_MAX: f32 = 2.50;
pub const TOUCH_HEIGHT_JUGGLE_MIN: f32 = 0.32;
pub const TOUCH_HEIGHT_JUGGLE_MAX: f32 = 1.45;
pub const TOUCH_GROUND_JUGGLE_HEIGHT_MAX: f32 = BALL_RADIUS + 0.10;

pub const TOUCH_FORWARD_IMPULSE_KICK: f32 = 8.8;
pub const TOUCH_FORWARD_IMPULSE_HEAD: f32 = 8.0;
pub const TOUCH_FORWARD_IMPULSE_JUGGLE: f32 = 0.8;
pub const TOUCH_UP_IMPULSE_KICK: f32 = 10.0;
pub const TOUCH_UP_IMPULSE_HEAD: f32 = 6.2;
pub const TOUCH_UP_IMPULSE_JUGGLE: f32 = 5.4;
pub const TOUCH_JUGGLE_HORIZONTAL_DAMP: f32 = 0.05;
pub const TOUCH_HEAD_VERTICAL_CUSHION: f32 = 0.35;

pub const BALL_RADIUS: f32 = 0.35;
pub const BALL_START_X: f32 = 0.0;
pub const BALL_START_Y: f32 = BALL_RADIUS;
pub const BALL_START_Z: f32 = 0.0;
pub const BALL_GRAVITY: f32 = 24.0;
pub const BALL_GROUND_BOUNCE: f32 = 0.45;
pub const BALL_WALL_BOUNCE: f32 = 0.70;
pub const BALL_AIR_DRAG: f32 = 0.25;
pub const BALL_GROUND_FRICTION: f32 = 5.5;
pub const BALL_MIN_BOUNCE_SPEED: f32 = 0.9;
pub const BALL_MAX_HORIZONTAL_SPEED: f32 = 14.0;
pub const BALL_PLAYER_CONTACT_HEIGHT_MAX: f32 = BALL_RADIUS + 0.18;
pub const BALL_PLAYER_CONTACT_DAMP: f32 = 0.35;
pub const BALL_PLAYER_WALK_PUSH: f32 = 0.10;
pub const BALL_PLAYER_REST_SPEED: f32 = 0.25;

pub const ELEGANCE_MAX: f32 = 100.0;
pub const ELEGANCE_PANIC_HEIGHT: f32 = 0.45;
pub const ELEGANCE_TOO_HIGH_HEIGHT: f32 = 2.7;
