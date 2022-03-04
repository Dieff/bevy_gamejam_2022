use bevy::prelude::Color;
use bevy_egui::egui::Color32;

/// Contains various costants used throughout the game

// Colors
pub const MOVE_INDICATION_ARROW_COLOR: Color = Color::GREEN;
pub const ATTACK_INDICATION_ARROW_COLOR: Color = Color::RED;

pub const MOVE_SELECTOR_COLOR: Color = Color::rgba(51./255., 242./255., 34./255., 0.4);
pub const ATTACK_SELECTOR_COLOR: Color = Color::rgba(255./255., 234./255., 0., 0.4);

pub const GAME_TITLE_COLOR1: Color32 = Color32::from_rgb(3, 44, 252);
pub const GAME_TITLE_COLOR2: Color32 = Color32::from_rgb(192, 0, 245);
pub const GAME_TITLE_COLOR3: Color32 = Color32::from_rgb(196, 2, 2);
pub const GAME_TITLE_COLOR4: Color32 = Color32::from_rgb(10, 117, 0);

// Player data
pub const PLAYER_MAX_HEALTH: f32 = 100.;
pub const PLAYER_MAX_MAGIKA: f32 = 100.;
pub const PLAYER_DEFAULT_MOVE_SPEED: u32 = 4;
pub const PLAYER_ATTACK_DAMAGE: f32 = 50.;

// Enemy data
pub const ENEMY_DEFAULT_MOVE_SPEED: u32 = 3;
pub const ENEMY_ATTACK_DAMAGE: f32 = 10.;

// Z levels
// map tiles are drawn at z level 1.
pub const MAP_UI_Z_LEVEL: f32 = 2.;
pub const MAP_INDICATOR_Z_LEVEL: f32 = 3.;
pub const PLAYER_Z_LEVEL: f32 = 4.;

// In-game UI
pub const TOP_BAR_MIN_SIZE: f32 = 30.;
pub const TOP_BAR_DESIRED_SIZE: f32 = 0.1;

// other
pub const TILE_SIZE: f32 = 16.; // don't change unless LDtk maps are updated
// how many seconds the turn animation lasts
pub const TURN_ANIMATION_DURATION: f32 = 1.;
pub const GAME_NAME: &'static str = "Space Wizard Power Tournament";
pub const MAP_SCROLL_FACTOR: f32 = 0.35;
