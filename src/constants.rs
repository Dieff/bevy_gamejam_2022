use bevy::prelude::Color;

/// Contains various costants used elsewhere

// Colors
pub const ACTION_INDICATION_ARROW_COLOR: Color = Color::rgba(0.01, 0.82, 0.95, 0.5);

// Player data
pub const PLAYER_MAX_HEALTH: f32 = 100.;
pub const PLAYER_MAX_MAGIKA: f32 = 100.;
pub const PLAYER_DEFAULT_MOVE_SPEED: u32 = 4;

// Enemy data



// Z levels
// tiles are drawn at z level 1.
pub const MAP_UI_Z_LEVEL: f32 = 2.;
pub const MAP_INDICATOR_Z_LEVEL: f32 = 3.;
pub const PLAYER_Z_LEVEL: f32 = 4.;

// In-game UI
pub const TOP_BAR_MIN_SIZE: f32 = 20.;
pub const TOP_BAR_DESIRED_SIZE: f32 = 0.1;

// other
// don't change this unless the ldtk maps are updated
pub const TILE_SIZE: f32 = 16.;

pub const TURN_ANIMATION_DURATION: f32 = 2.;

pub const GAME_NAME: &'static str = "Space Wizard Power Tournament";
