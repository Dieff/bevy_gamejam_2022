use bevy::prelude::*;
use bevy_ecs_tilemap::TilePos;

use crate::constants;
use crate::map_entities::{player::PlayerStatus, EntityHealth, MapEntityType};
use crate::utils;
use crate::GameState;

mod animation;
pub use animation::{EnemyTurnAnimating, PlayerTurnAnimating, TurnDisplayer};
mod map_ui;

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAttack {
  enemy_entity: Entity,
  enemy_position: TilePos,
  new_standing_position: TilePos,
  starting_position: TilePos,
}

impl PendingAttack {
  pub fn new(
    enemy_entity: Entity,
    enemy_position: TilePos,
    new_standing_position: TilePos,
    starting_position: TilePos,
  ) -> Self {
    Self {
      enemy_entity,
      enemy_position,
      new_standing_position,
      starting_position,
    }
  }
}

/// The possible things an entity can do in a turn
#[derive(Debug, Clone, PartialEq)]
pub enum EntityAction {
  Move((TilePos, TilePos)),
  Attack(PendingAttack),
  Wait,
  Cast(String),
}

/// Marker component for entities that do things during turns
/// such as players or enemies
#[derive(Component, Debug, Default)]
pub struct MapEntity;

/// These components are created before a turn is exectured (as their own entities)
/// They are removed at the end of the turn.
#[derive(Component, Clone, Debug)]
pub struct EntityPendingAction {
  pub action: EntityAction,
  pub is_ready: bool,
}

impl EntityPendingAction {
  pub fn is_move(&self) -> bool {
    if let EntityAction::Move(_) = self.action {
      true
    } else {
      false
    }
  }
  pub fn is_attack(&self) -> bool {
    if let EntityAction::Attack(_) = self.action {
      true
    } else {
      false
    }
  }
  pub fn is_wait(&self) -> bool {
    self.action == EntityAction::Wait
  }
  pub fn default_attack() -> Self {
    Self {
      is_ready: false,
      action: EntityAction::Attack(PendingAttack::new(
        Entity::from_raw(0), // be sure not to try and use this
        Default::default(),
        Default::default(),
        Default::default(),
      )),
    }
  }
  pub fn get_spell<'a>(&'a self) -> Option<&'a str> {
    if let EntityAction::Cast(spell) = &self.action {
      Some(spell.as_str())
    } else {
      None
    }
  }
}

impl Default for EntityPendingAction {
  fn default() -> Self {
    EntityPendingAction {
      action: EntityAction::Wait,
      is_ready: true,
    }
  }
}

/// An event that is sent by the UI to indicated a player's action has been chosen
pub struct PlayerActionChosen {
  pub action_type: EntityPendingAction,
  pub player: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TurnStatus {
  PlayerRunning,
  EnemyRunning,
  PlayerChoosing,
}

/// Event used when we need to execute a new turn
#[derive(Debug)]
pub struct StartTurn(pub TurnStatus);

/// Event used when the turn is over, and the player is choosing once again
pub struct EndTurn;

// a resource that records the current turn phase
pub struct TurnUIState {
  pub status: TurnStatus,
}

impl TurnUIState {
  pub fn new() -> Self {
    Self {
      status: TurnStatus::PlayerChoosing,
    }
  }
}

/// Created as a seperate entity every time a turn cycle has finished
/// so we can keep track of what has happened.
#[derive(Component)]
pub struct CompletedTurn {
  wizard_health: f32,
  warrior_health: f32,
}

/// Fires at the end of a turn animation
/// has to update the state/positions of map entities
pub fn execute_turn(
  player_turn_ended: RemovedComponents<PlayerTurnAnimating>,
  enemy_turn_ended: RemovedComponents<EnemyTurnAnimating>,
  mut entity_q: Query<(&mut TilePos, &MapEntityType, &mut EntityPendingAction)>,
  mut health: Query<&mut EntityHealth>,
) {
  let has_player_turn_ended = player_turn_ended.iter().count() > 0;
  let has_enemy_turn_ended = enemy_turn_ended.iter().count() > 0;

  if !has_player_turn_ended && !has_enemy_turn_ended {
    return;
  }

  entity_q
    .iter_mut()
    .filter(|(_, kind, _)| {
      if let MapEntityType::Player(_) = kind {
        has_player_turn_ended
      } else {
        has_enemy_turn_ended
      }
    })
    .for_each(|(mut pos, entity_type, mut action)| {
      match action.action {
        EntityAction::Move((_, end)) => {
          pos.0 = end.0;
          pos.1 = end.1;
        }
        EntityAction::Attack(PendingAttack {
          enemy_entity: target_entity,
          new_standing_position: new_pos,
          enemy_position: target_pos,
          ..
        }) => {
          // TODO: sometimes this fails
          if utils::tile_distance(&new_pos, &target_pos) == 1 {
            // Todo: the damage amount should be contained in `PendingAttack`
            let damage = match entity_type {
              &MapEntityType::Enemy => constants::ENEMY_ATTACK_DAMAGE,
              &MapEntityType::Player(_) => constants::PLAYER_ATTACK_DAMAGE,
              _ => 0.,
            };

            if let Ok(mut health) = health.get_mut(target_entity) {
              health.health -= damage;
            }
          }
          pos.0 = new_pos.0;
          pos.1 = new_pos.1;
        }
        _ => {}
      }

      action.is_ready = true;
      action.action = EntityAction::Wait;
    });
}

/// Fires after the turn animation has completed.
/// In charge of setting the correct state so that the player can choose their next action
pub fn start_player_choice_phase(
  mut commands: Commands,
  mut turn_event: EventReader<EndTurn>,
  mut turn_ui_state: ResMut<TurnUIState>,
  player_health: Query<(&EntityHealth, &MapEntityType), With<PlayerStatus>>,
) {
  if turn_event.iter().count() != 0 {
    turn_ui_state.status = TurnStatus::PlayerChoosing;

    let mut ct = CompletedTurn {
      warrior_health: 0.,
      wizard_health: 0.,
    };
    for (h, pt) in player_health.iter() {
      if let MapEntityType::Player(crate::map_entities::PlayerType::Warrior) = pt {
        ct.warrior_health = h.health;
      } else {
        ct.wizard_health = h.health;
      }
    }

    commands.spawn().insert(ct);
  }
}

/// The UI sends PlayerActionChosen events when an action is added, removed, or updated
/// Here, we set the proper EntityPendingAction on the player entity
pub fn update_player_action(
  mut commands: Commands,
  mut new_actions: EventReader<PlayerActionChosen>,
) {
  for PlayerActionChosen {
    action_type: new_action,
    player,
  } in new_actions.iter()
  {
    commands.entity(*player).remove::<EntityPendingAction>();
    commands.entity(*player).insert(new_action.to_owned());
  }
}

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(TurnUIState::new())
      .add_event::<StartTurn>()
      .add_event::<EndTurn>()
      .add_event::<PlayerActionChosen>()
      .add_plugin(animation::TurnAnimationPlugin)
      .add_system_set(
        SystemSet::on_update(GameState::Running)
          .with_system(start_player_choice_phase)
          // The order here was found by trial and error to avoid a tricky race condition.
          // TODO: clear up race condition
          .with_system(update_player_action.before("player-move-selector"))
          .with_system(map_ui::remove_action_arrows.before("draw-action-arrows"))
          .with_system(map_ui::draw_action_arrows.label("draw-action-arrows"))
          .with_system(map_ui::spawn_action_choosers.label("player-move-selector"))
          .with_system(map_ui::despawn_action_choosers.after("player-move-selector"))
          .with_system(map_ui::select_player_move)
          .with_system(map_ui::select_player_attack),
      )
      .add_system_set(
        SystemSet::on_exit(GameState::Running).with_system(map_ui::despawn_ui_elements),
      )
      // We have to run this system after the update because it is looking for removed components,
      // information about which is only retained for one frame.
      .add_system_to_stage(CoreStage::PostUpdate, execute_turn);
  }
}
