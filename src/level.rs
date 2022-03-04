use bevy::prelude::*;

use crate::{
  map_entities::{enemy::Enemy, player::PlayerStatus, EntityHealth},
  turn::CompletedTurn,
  GameState,
};

#[derive(Component)]
pub struct AvailableLevel {
  pub name: String,
  pub ldtk_id: usize,
}

impl From<&(&'static str, usize)> for AvailableLevel {
  fn from(d: &(&'static str, usize)) -> Self {
    Self {
      name: d.0.to_owned(),
      ldtk_id: d.1,
    }
  }
}

// TODO: in an ideal world, this information would be stored seperately
pub fn level_startup(mut commands: Commands) {
  [("Test Level", 0)].iter().for_each(|d| {
    let av = AvailableLevel::from(d);
    commands.spawn().insert(av);
  })
}

/// An event that fires when the user goes to the level select menu
pub struct ToMenu;

pub enum RoundResult {
  Victory,
  Defeat,
  Neutral,
}

#[derive(Component)]
pub struct RoundSummary(pub RoundResult);

/// add to an AvailableLevel to trigger level loading
#[derive(Component)]
pub struct CurrentLevel;

/// add to an AvailableLevel to show that it has been beaten
#[derive(Component)]
pub struct CompletedLevel;

pub fn end_round(
  mut commands: Commands,
  player_health: Query<&EntityHealth, (With<PlayerStatus>, Changed<EntityHealth>)>,
  enemy_health: Query<&EntityHealth, (With<Enemy>, Changed<EntityHealth>)>,
  cur_level: Query<Entity, With<CurrentLevel>>,
) {
  if player_health.iter().any(|ph| ph.health == 0.) {
    commands.spawn().insert(RoundSummary(RoundResult::Defeat));
    return;
  }
  if !enemy_health.is_empty() && enemy_health.iter().all(|eh| eh.health == 0.) {
    commands.spawn().insert(RoundSummary(RoundResult::Victory));
    if let Ok(cl) = cur_level.get_single() {
      commands.entity(cl).insert(CompletedLevel);
    }
  }
}

pub fn return_to_menu(
  mut commands: Commands,
  mut events: EventReader<ToMenu>,
  mut game_state: ResMut<State<GameState>>,
  turns: Query<Entity, With<CompletedTurn>>,
  round_result: Query<Entity, With<RoundSummary>>,
  cur_level: Query<Entity, With<CurrentLevel>>,
) {
  let mut do_return = false;
  for _ in events.iter() {
    do_return = true;
  }

  if !do_return {
    return;
  }
  game_state.replace(GameState::Menu).unwrap();

  for e in turns.iter().chain(round_result.iter()) {
    commands.entity(e).despawn_recursive();
  }

  if let Ok(le) = cur_level.get_single() {
    commands.entity(le).remove::<CurrentLevel>();
  }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_startup_system(level_startup)
      .add_system(end_round)
      .add_system(return_to_menu)
      .add_event::<ToMenu>();
  }
}
