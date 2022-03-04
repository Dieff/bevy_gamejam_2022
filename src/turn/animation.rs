use bevy::prelude::*;
use bevy_ecs_tilemap::TileSize;

use super::{
  EndTurn, EntityAction, EntityPendingAction, PendingAttack, StartTurn, TurnStatus, TurnUIState,
};
use crate::{constants, map_entities::MapEntityType, utils};

#[derive(Component)]
pub struct PlayerTurnAnimating;

#[derive(Component)]
pub struct EnemyTurnAnimating;

/// One of these is created as it's own entity while a turn is being displayed
#[derive(Component)]
pub struct TurnDisplayer {
  timer: Timer,
}

/// In charge of setting up the state to animate a turn
pub fn create_turn_animations(
  mut commands: Commands,
  mut turn_events: EventReader<StartTurn>,
  mut ui_state: ResMut<TurnUIState>,
) {
  if let Some(turn) = turn_events.iter().next() {
    let timer = Timer::from_seconds(constants::TURN_ANIMATION_DURATION, false);
    let displayer = TurnDisplayer { timer };
    match turn.0 {
      TurnStatus::PlayerRunning => {
        commands
          .spawn()
          .insert(displayer)
          .insert(PlayerTurnAnimating);
        ui_state.status = TurnStatus::PlayerRunning;
      }
      _ => {
        commands
          .spawn()
          .insert(displayer)
          .insert(EnemyTurnAnimating);
        ui_state.status = TurnStatus::EnemyRunning;
      }
    }
  }
  assert!(turn_events.iter().count() == 0);
}

/// In charge of updating/removing the TurnDisplayer struct
pub fn update_turn_animations(
  mut commands: Commands,
  time: ResMut<Time>,
  mut turn_phase_events: EventWriter<StartTurn>,
  mut turn_end_events: EventWriter<EndTurn>,
  mut player_animation_q: Query<
    (Entity, &mut TurnDisplayer),
    (With<PlayerTurnAnimating>, Without<EnemyTurnAnimating>),
  >,
  mut enemy_animation_q: Query<
    (Entity, &mut TurnDisplayer),
    (With<EnemyTurnAnimating>, Without<PlayerTurnAnimating>),
  >,
) {
  let is_player_animating = !player_animation_q.is_empty();
  let is_enemy_animation = !enemy_animation_q.is_empty();

  // can't do both at once
  assert!(!(is_player_animating && is_enemy_animation));

  if let Ok((e, mut displayer)) = player_animation_q
    .get_single_mut()
    .or(enemy_animation_q.get_single_mut())
  {
    displayer.timer.tick(time.delta());
    if displayer.timer.finished() {
      commands.entity(e).despawn_recursive();
      if is_player_animating {
        turn_phase_events.send(StartTurn(TurnStatus::EnemyRunning));
      } else if is_enemy_animation {
        turn_end_events.send(EndTurn);
      } else {
        panic!("TurnDisplayer component exists without marker!");
      }
    }
  }
}

pub fn run_move_animations(
  turn_q: Query<(
    &TurnDisplayer,
    Option<&PlayerTurnAnimating>,
    Option<&EnemyTurnAnimating>,
  )>,
  mut map_entities: Query<(&mut Transform, &MapEntityType, &EntityPendingAction)>,
) {
  if let Ok((displayer, pt, et)) = turn_q.get_single() {
    let animation_ratio =
      displayer.timer.elapsed().as_millis() as f32 / displayer.timer.duration().as_millis() as f32;
    map_entities
      .iter_mut()
      .filter_map(|(transform, kind, action)| {
        if let MapEntityType::Player(_) = kind {
          if et.is_some() {
            return None;
          }
        }
        if let MapEntityType::Enemy = kind {
          if pt.is_some() {
            return None;
          }
        }

        match action.action {
          EntityAction::Move(mov) => Some((transform, mov)),
          EntityAction::Attack(PendingAttack {
            starting_position: start,
            new_standing_position: end,
            ..
          }) => Some((transform, (start, end))),
          _ => None,
        }
      })
      .for_each(|(mut transform, (start, end))| {
        let tile_size = TileSize(constants::TILE_SIZE, constants::TILE_SIZE);
        let start_world_point = utils::world_pos_from_tile_pos(&start, &tile_size);
        let end_world_point = utils::world_pos_from_tile_pos(&end, &tile_size);
        let dp = end_world_point - start_world_point;
        transform.translation.x = start_world_point.x + dp.x * animation_ratio;
        transform.translation.y = start_world_point.y + dp.y * animation_ratio;
      })
  }
}

pub struct TurnAnimationPlugin;

impl Plugin for TurnAnimationPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_system(create_turn_animations)
      .add_system(update_turn_animations)
      .add_system(run_move_animations);
  }
}
