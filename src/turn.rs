use bevy::prelude::*;
use bevy_ecs_tilemap::TilePos;

pub enum PlayerAction {
  Move((TilePos, TilePos)),
  Attack,
  Wait,
  Cast(String)
}

/// Used when we need to execute a new turn
pub struct StartPlayerTurn;

pub fn do_turn(mut event: EventReader<StartPlayerTurn>, mut memory: ResMut<crate::wizard_memory::MemoryBlob>) {
  if event.iter().count() > 0 {
    // run the turn
    memory.reset_player_memory();
  }
}

