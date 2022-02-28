use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::{TileParent, TilePos, TileSize};

use crate::map::{self, DataLayer};

#[derive(Debug, Clone, Copy)]
enum PlayerType {
  Wizard,
  Warrior,
}

#[derive(Debug, Component, Clone)]
pub struct PlayerIdentity {
  player: PlayerType,
}

impl Default for PlayerIdentity {
  fn default() -> Self {
    Self {
      player: PlayerType::Wizard,
    }
  }
}

impl From<EntityInstance> for PlayerIdentity {
  fn from(instance: EntityInstance) -> Self {
    match instance.identifier.as_str() {
      "Player1Start" => Self {
        player: PlayerType::Wizard,
      },
      "Player2Start" => Self {
        player: PlayerType::Warrior,
      },
      _ => panic!("Invalid PlayerStart identifier"),
    }
  }
}

#[derive(Component)]
pub struct PlayerStatus {
  pub health: f32,
}

impl Default for PlayerStatus {
  fn default() -> Self {
    Self { health: 10. }
  }
}

#[derive(Bundle, LdtkEntity)]
pub struct PlayerStartBundle {
  start: PlayerIdentity,
  #[grid_coords]
  pos: GridCoords,
}

#[derive(Bundle, Default)]
struct NewPlayerBundle {
  status: PlayerStatus,
  identity: PlayerIdentity,
  om: map::OnMap,
  map_pos: TilePos,
  /// Refers to the tile in the data layer on which the player is sitting
  parent: TileParent,
  #[bundle]
  sprite: SpriteBundle,
}

pub fn spawn_players_on_map(
  mut commands: Commands,
  player_start: Query<(&GridCoords, &PlayerIdentity), Added<PlayerIdentity>>,
  data_tiles: Query<(Entity, &TilePos, &TileParent), With<DataLayer>>,
) {
  for (tile_entity, tile_pos, tile_parent) in data_tiles.iter() {
    for (coords, player) in player_start.iter() {
      if coords.x as u32 == tile_pos.0 && coords.y as u32 == tile_pos.1 {
        commands.spawn_bundle(NewPlayerBundle {
          map_pos: tile_pos.to_owned(),
          parent: tile_parent.to_owned(),
          identity: player.to_owned(),
          sprite: crate::utils::new_square_sprite_bundle(
            Color::rgb(1., 1., 1.),
            Vec2::new(16., 16.),
            crate::utils::initial_map_drawing_position(&TileSize(16., 16.), tile_pos),
          ),
          ..Default::default()
        });
        commands.entity(tile_entity).insert(map::TileHasEntity);
      }
    }
  }
}
