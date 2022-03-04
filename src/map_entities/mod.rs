use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::{TileParent, TilePos, TileSize};

use crate::{
  constants,
  map::{self, DataLayer},
  turn::EntityPendingAction,
};

pub mod player;
pub mod enemy;

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
      "EnemyStart" => Self::Enemy,
      _ => panic!("Invalid PlayerStart identifier"),
    }
  }
}

#[derive(Component, Debug)]
pub struct EntityHealth {
  pub health: f32,
}

impl Default for EntityHealth {
  fn default() -> Self {
    Self {
      health: constants::PLAYER_MAX_HEALTH,
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


pub fn spawn_entities_on_map(
  mut commands: Commands,
  entity_start: Query<(&GridCoords, &MapEntityType), Added<MapEntityType>>,
  data_tiles: Query<(Entity, &TilePos, &TileParent), With<DataLayer>>,
) {
  for (tile_entity, tile_pos, tile_parent) in data_tiles.iter() {
    for (coords, entity_type) in entity_start.iter() {
      if coords.x as u32 == tile_pos.0 && coords.y as u32 == tile_pos.1 {
        match entity_type {
          &MapEntityType::Player(player_id) => {
            let is_wizard = player_id == PlayerType::Wizard;

            commands.spawn_bundle(player::NewPlayerBundle {
              map_pos: tile_pos.to_owned(),
              parent: tile_parent.to_owned(),
              map_entity_type: entity_type.to_owned(),
              sprite: crate::utils::new_square_sprite_bundle(
                Color::rgb(1., 1., 1.),
                Vec2::new(16., 16.),
                crate::utils::initial_map_drawing_position(
                  &TileSize(16., 16.),
                  tile_pos,
                  constants::PLAYER_Z_LEVEL,
                ),
              ),
              status: player::PlayerStatus {
                magika: if is_wizard {
                  Some(constants::PLAYER_MAX_MAGIKA)
                } else {
                  None
                },
                ..Default::default()
              },
              ..Default::default()
            });
            commands.entity(tile_entity).insert(map::TileHasEntity);
          }
          &MapEntityType::Enemy => {
            commands.spawn_bundle(enemy::NewEnemyBundle {
              map_pos: tile_pos.to_owned(),
              parent: tile_parent.to_owned(),
              map_entity_type: entity_type.to_owned(),
              sprite: crate::utils::new_square_sprite_bundle(
                Color::RED,
                Vec2::new(16., 16.),
                crate::utils::initial_map_drawing_position(
                  &TileSize(16., 16.),
                  tile_pos,
                  constants::PLAYER_Z_LEVEL,
                ),
              ),
              ..Default::default()
            });
          }
          _ => {}
        }
      }
    }
  }
}

pub struct MapEntityPlugin;

impl Plugin for MapEntityPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_ldtk_entity::<MapEntityStart>("Player1Start")
      .register_ldtk_entity::<MapEntityStart>("Player2Start")
      .register_ldtk_entity::<MapEntityStart>("EnemyStart")
      .add_system(spawn_entities_on_map)
      .add_system(enemy::enemy_ai);
  }
}
