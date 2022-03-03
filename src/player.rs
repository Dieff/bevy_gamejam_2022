use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::{TileParent, TilePos, TileSize};

use crate::{
  constants,
  map::{self, DataLayer},
};

#[derive(Debug, Component, Clone, Copy, PartialEq, Hash)]
pub enum PlayerType {
  Wizard,
  Warrior,
}

#[derive(Debug, Component, Clone, PartialEq)]
pub enum MapEntityType {
  Player(PlayerType),
  Enemy,
  Neutral,
}

impl Default for MapEntityType {
  fn default() -> Self {
    Self::Neutral
  }
}

impl From<EntityInstance> for MapEntityType {
  fn from(instance: EntityInstance) -> Self {
    match instance.identifier.as_str() {
      "Player1Start" => Self::Player(PlayerType::Wizard),
      "Player2Start" => Self::Player(PlayerType::Warrior),
      _ => panic!("Invalid PlayerStart identifier"),
    }
  }
}

#[derive(Component)]
pub struct PlayerStatus {
  pub health: f32,
  pub magika: Option<f32>,
}

impl Default for PlayerStatus {
  fn default() -> Self {
    Self {
      health: constants::PLAYER_MAX_HEALTH,
      magika: None,
    }
  }
}

#[derive(Bundle, LdtkEntity)]
pub struct MapEntityStart {
  #[from_entity_instance]
  start: MapEntityType,
  #[grid_coords]
  pos: GridCoords,
}

#[derive(Bundle, Default)]
struct NewPlayerBundle {
  status: PlayerStatus,
  //identity: PlayerIdentity,
  map_entity_type: MapEntityType,
  om: map::DrawOnMap,
  map_pos: TilePos,
  /// Refers to the tile in the data layer on which the player is sitting
  parent: TileParent,
  #[bundle]
  sprite: SpriteBundle,
}

pub fn spawn_players_on_map(
  mut commands: Commands,
  player_start: Query<(&GridCoords, &MapEntityType), Added<MapEntityType>>,
  data_tiles: Query<(Entity, &TilePos, &TileParent), With<DataLayer>>,
) {
  for (tile_entity, tile_pos, tile_parent) in data_tiles.iter() {
    for (coords, player_type) in player_start.iter() {
      if coords.x as u32 == tile_pos.0 && coords.y as u32 == tile_pos.1 {
        let is_wizard = *player_type == MapEntityType::Player(PlayerType::Wizard);

        commands
          .spawn_bundle(NewPlayerBundle {
            map_pos: tile_pos.to_owned(),
            parent: tile_parent.to_owned(),
            map_entity_type: player_type.to_owned(),
            sprite: crate::utils::new_square_sprite_bundle(
              Color::rgb(1., 1., 1.),
              Vec2::new(16., 16.),
              crate::utils::initial_map_drawing_position(
                &TileSize(16., 16.),
                tile_pos,
                constants::PLAYER_Z_LEVEL,
              ),
            ),
            status: PlayerStatus {
              magika: if is_wizard {
                Some(constants::PLAYER_MAX_MAGIKA)
              } else {
                None
              },
              ..Default::default()
            },
            ..Default::default()
          }) // We expect the players to have an action by default
          .insert(crate::turn::EntityPendingAction {
            is_ready: true,
            action: crate::turn::EntityAction::Wait,
          });
        commands.entity(tile_entity).insert(map::TileHasEntity);
      }
    }
  }
}
