use bevy::prelude::*;
use bevy_ecs_tilemap::{TileParent, TilePos};

use crate::{
  map::{self},
  turn::EntityPendingAction,
};

use super::{EntityHealth, MapEntityType};

#[derive(Component)]
pub struct PlayerStatus {
  pub magika: Option<f32>,
}

impl Default for PlayerStatus {
  fn default() -> Self {
    Self { magika: None }
  }
}

#[derive(Bundle, Default)]
pub struct NewPlayerBundle {
  pub status: PlayerStatus,
  pub health: EntityHealth,
  pub map_entity_type: MapEntityType,
  pub om: map::DrawOnMap,
  pub map_pos: TilePos,
  /// Refers to the tile in the data layer on which the player is sitting
  pub parent: TileParent,
  #[bundle]
  pub sprite: SpriteBundle,
  pub action: EntityPendingAction,
}
