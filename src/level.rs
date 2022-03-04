use bevy::prelude::*;

use crate::map_entities::{enemy::Enemy, player::PlayerStatus, EntityHealth};

/// An event that fires when the user goes to the level select menu
pub struct ToLevelSelectMenu;

pub enum RoundResult {
  Victory,
  Defeat,
  Neutral,
}

#[derive(Component)]
pub struct RoundSummary(pub RoundResult);

pub fn end_round(
  mut commands: Commands,
  player_health: Query<&EntityHealth, (With<PlayerStatus>, Changed<EntityHealth>)>,
  enemy_health: Query<&EntityHealth, (With<Enemy>, Changed<EntityHealth>)>,
) {
  if player_health.iter().any(|ph| ph.health == 0.) {
    commands.spawn().insert(RoundSummary(RoundResult::Defeat));
    return;
  }
  if !enemy_health.is_empty() && enemy_health.iter().all(|eh| eh.health == 0.) {
    commands.spawn().insert(RoundSummary(RoundResult::Victory));
  }
}

pub fn return_to_menu(mut commands: Commands, events: EventReader<ToLevelSelectMenu>) {
  
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
  fn build(&self, app: &mut App) {
    app.add_system(end_round).add_event::<ToLevelSelectMenu>();
  }
}
