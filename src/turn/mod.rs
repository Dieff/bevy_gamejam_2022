use bevy::prelude::*;
use bevy_ecs_tilemap::{MapQuery, TileParent, TilePos, TileSize};
use bevy_prototype_lyon::prelude::*;

use crate::map::{DataLayer, DrawOnMap, TileKind, TileSelectedEvent};
use crate::player::{MapEntityType, PlayerStatus, PlayerType};
use crate::spells;
use crate::utils;
use crate::{constants};

/// The possible things an entity can do in a turn
#[derive(Debug, Clone, PartialEq)]
pub enum EntityAction {
  Move((TilePos, TilePos)),
  Attack,
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
  pub fn wait() -> Self {
    Self {
      action: EntityAction::Wait,
      is_ready: true,
    }
  }
  pub fn move_default() -> Self {
    Self {
      action: EntityAction::Move((TilePos::default(), TilePos::default())),
      is_ready: false,
    }
  }
  pub fn is_move(&self) -> bool {
    if let EntityAction::Move(_) = self.action {
      true
    } else {
      false
    }
  }
  pub fn is_wait(&self) -> bool {
    self.action == EntityAction::Wait
  }
  pub fn get_spell<'a>(&'a self) -> Option<&'a str> {
    if let EntityAction::Cast(spell) = &self.action {
      Some(spell.as_str())
    }
    else {
      None
    }
  }
}

/// An event that is sent by the UI to indicated a player's action has been chosen
pub struct PlayerActionChosen {
  pub action_type: EntityPendingAction,
  pub player: Entity,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TurnStatus {
  PlayerRunning,
  EnemyRunning,
  PlayerChoosing,
  None,
}

#[derive(Component)]
pub struct TurnDisplayer {
  timer: Timer,
  kind: TurnStatus,
}

/// Event used when we need to execute a new turn
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
pub struct CompletedTurn;

/// Handles events that single a new turn phase.
/// If the turn is animating (`TurnStates::PlayerRunning` or `TurnStatus::EnemyRunning`),
/// a `TurnDisplayer` component will be spawned.
pub fn start_turn(
  mut commands: Commands,
  mut turn_event: EventReader<StartTurn>,
  mut new_player_choose_phase: EventWriter<PlayerActionChosen>,
  mut memory: ResMut<spells::wizard_memory::MemoryBlob>,
  mut turn_ui_state: ResMut<TurnUIState>,
  players: Query<Entity, With<PlayerStatus>>,
) {
  if let Some(new_turn_state) = turn_event.iter().next() {
    turn_ui_state.status = new_turn_state.0;
    if new_turn_state.0 == TurnStatus::PlayerRunning {
      memory.reset_player_memory();
      commands.spawn().insert(TurnDisplayer {
        timer: Timer::from_seconds(constants::TURN_ANIMATION_DURATION, false),
        kind: TurnStatus::PlayerRunning,
      });
    }
    if new_turn_state.0 == TurnStatus::EnemyRunning {
      commands.spawn().insert(TurnDisplayer {
        timer: Timer::from_seconds(constants::TURN_ANIMATION_DURATION, false),
        kind: TurnStatus::EnemyRunning,
      });
    }

    for entity in players.iter() {
      commands.entity(entity).remove::<DrawOnMap>();
    }
  }
}

pub fn start_player_choice(
  mut commands: Commands,
  mut turn_event: EventReader<EndTurn>,
  mut entity_q: Query<(Entity, &mut EntityPendingAction)>,
) {
  if turn_event.iter().count() != 0 {
    for (entity, mut pending_action) in entity_q.iter_mut() {
      pending_action.is_ready = true;
      pending_action.action = EntityAction::Wait;
      commands.entity(entity).insert(DrawOnMap);
    }
    commands.spawn().insert(CompletedTurn);
  }
}

pub fn run_turn_animation(
  mut commands: Commands,
  time: ResMut<Time>,
  mut event_writer: EventWriter<StartTurn>,
  mut end_writer: EventWriter<EndTurn>,
  mut turns: Query<(Entity, &mut TurnDisplayer)>,
  actions: Query<(Entity, &EntityPendingAction)>,
) {
  if let Ok((turn_entity, mut turn)) = turns.get_single_mut() {
    turn.timer.tick(time.delta());
    if turn.timer.finished() {
      commands.entity(turn_entity).despawn_recursive();
      let next_phase = match turn.kind {
        TurnStatus::PlayerRunning => TurnStatus::EnemyRunning,
        TurnStatus::EnemyRunning => TurnStatus::PlayerChoosing,
        _ => TurnStatus::None,
      };
      if next_phase == TurnStatus::PlayerChoosing {
        dbg!("A turn cycle has completed");
        end_writer.send(EndTurn);
      }
      event_writer.send(StartTurn(next_phase));
      for (entity, _) in actions.iter() {
        commands.entity(entity).remove::<EntityPendingAction>();
      }
      return;
    }
  }
}

#[derive(Component, Debug)]
pub struct ActionIndicationMarker;

/// Inserts action arrows for pending player actions
pub fn draw_action_arrows(
  mut commands: Commands,
  pending: Query<&EntityPendingAction, Changed<EntityPendingAction>>,
  active_turn: Query<&TurnDisplayer>,
) {
  for action in pending.iter() {
    if action.is_ready && active_turn.is_empty() {
      if let EntityAction::Move((start, end)) = action.action {
        // a new pending action has been added, but the turn is not yet animating
        // so, let's draw an arrow to show the intended action
        let start_point = utils::initial_map_drawing_position(
          &TileSize(constants::TILE_SIZE, constants::TILE_SIZE),
          &start,
          0.,
        )
        .truncate();
        let end_point = utils::initial_map_drawing_position(
          &TileSize(constants::TILE_SIZE, constants::TILE_SIZE),
          &end,
          0.,
        )
        .truncate();

        // Create an arrow
        let hyp = end_point - start_point;
        if hyp.x == 0. && hyp.y == 0. {
          return;
        }
        let angle = hyp.angle_between(Vec2::X).abs();

        let upper_angle = angle + 0.4;
        let lower_angle = angle - 0.4;
        let mut upper_vec =
          Vec2::new(1., upper_angle.tan()).normalize() * (constants::TILE_SIZE - 2.);
        let mut lower_vec =
          Vec2::new(1., lower_angle.tan()).normalize() * (constants::TILE_SIZE - 2.);
        if hyp.x < 0. && hyp.y >= 0. {
          upper_vec = upper_vec * -1.;
          lower_vec = lower_vec * -1.;
        } else if hyp.y < 0. && hyp.x > 0. {
          upper_vec.y = upper_vec.y * -1.;
          lower_vec.y = lower_vec.y * -1.;
        } else if hyp.y < 0. && hyp.x < 0. {
          upper_vec.x = upper_vec.x * -1.;
          lower_vec.x = lower_vec.x * -1.;
        } else if hyp.y < 0. && hyp.x == 0. {
          lower_vec = lower_vec * -1.;
        } else if hyp.x == 0. && hyp.y > 0. {
          upper_vec = upper_vec * -1.;
        }
        let upper_ch_end = end_point - upper_vec;
        let lower_ch_end = end_point - lower_vec;

        let arrow_line = shapes::Line(start_point, end_point);
        let upper_chevron = shapes::Line(end_point, upper_ch_end);
        let lower_chevron = shapes::Line(end_point, lower_ch_end);

        let mut transform: Transform = Default::default();
        transform.translation.z = constants::MAP_INDICATOR_Z_LEVEL;

        // The arrow indicators are their own entities, rather than being attached to players,
        // because they have some duplicate components with the player's sprites.
        commands
          .spawn_bundle(
            GeometryBuilder::new()
              .add(&arrow_line)
              .add(&upper_chevron)
              .add(&lower_chevron)
              .build(
                DrawMode::Stroke(StrokeMode::new(
                  constants::ACTION_INDICATION_ARROW_COLOR,
                  3.,
                )),
                transform,
              ),
          )
          .insert(ActionIndicationMarker)
          .insert(start.clone());
      }
    }
  }
}

pub fn remove_action_arrows(
  mut commands: Commands,
  player_q: Query<(&TilePos, &EntityPendingAction), With<PlayerStatus>>,
  arrow_q: Query<(Entity, &TilePos), With<ActionIndicationMarker>>,
) {
  for (player_pos, player_pending_action) in player_q.iter() {
    if !player_pending_action.is_move() || !player_pending_action.is_ready {
      arrow_q
        .iter()
        .filter(|pos| pos.1 == player_pos)
        .for_each(|(e, _)| {
          commands.entity(e).despawn_recursive();
        })
    }
  }
}

// Components to help choose actions on the map
#[derive(Component)]
pub struct MapActionChooser {
  origin: TilePos,
  player: PlayerType,
}

#[derive(Component)]
pub struct PlayerMoveSelect;

#[derive(Component)]
pub struct PlayerAttackSelect;

// be sure to insert the correct marker component as well
#[derive(Bundle)]
pub struct MapActionChooserBundle {
  p: MapActionChooser,
  #[bundle]
  sprite: SpriteBundle,
  pos: TilePos,
  parent: TileParent,
  onmap: DrawOnMap,
}

impl MapActionChooserBundle {
  fn new(
    pos: &TilePos,
    parent: &TileParent,
    player_pos: &TilePos,
    player_type: &MapEntityType,
  ) -> Self {
    Self {
      p: MapActionChooser {
        origin: player_pos.to_owned(),
        player: match player_type {
          MapEntityType::Player(pt) => pt.to_owned(),
          _ => panic!("invalid MapEntityType"),
        },
      },
      pos: pos.to_owned(),
      parent: parent.to_owned(),
      onmap: DrawOnMap,
      sprite: crate::utils::new_square_sprite_bundle(
        Color::rgba(0.5, 1., 0.3, 0.3),
        Vec2::new(16., 16.),
        crate::utils::initial_map_drawing_position(
          &TileSize(16., 16.),
          pos,
          constants::MAP_UI_Z_LEVEL,
        ),
      ),
    }
  }
}

/// If a player has been clicked during the PlayerChoose phase of a turn,
/// or "Move" has been selected in the left UI,
/// this will spawn markers to show possible moves.
pub fn create_player_move_selector(
  mut commands: Commands,
  mut select_events: EventReader<TileSelectedEvent>,
  tile_q: Query<(&TilePos, &TileParent, &DataLayer)>,
  mut player_q: Query<
    (
      &TilePos,
      &TileParent,
      &MapEntityType,
      &mut EntityPendingAction,
    ),
    With<PlayerStatus>,
  >,
  map_marker_q: Query<(Entity, &MapActionChooser)>,
  mut map_query: MapQuery,
) {
  // there really shouldn't be more than one tile click event per frame
  let click_event = select_events.iter().next();

  let mut build_markers: Option<(TilePos, TileParent, MapEntityType)> = None;

  // A new click has happened. Click events override UI state
  if let Some(TileSelectedEvent(Some(click_pos))) = click_event {
    if let Some((player_pos, player_tile_parent, player_type, mut player_pending_action)) =
      player_q.iter_mut().find(|dp| dp.0 == click_pos)
    {
      // The pending action is not yet ready, because we don't know which square we want to move to
      player_pending_action.is_ready = false;
      player_pending_action.action = EntityAction::Move((TilePos::default(), TilePos::default()));
      build_markers = Some((
        player_pos.to_owned(),
        player_tile_parent.to_owned(),
        player_type.to_owned(),
      ));

      // Update the other player. If they were busy selecting a move, they can't anymore
      if let Some((_, _, _, mut other_player_action)) = player_q
        .iter_mut()
        .find(|(pos, _, _, status)| *pos != click_pos && !status.is_ready)
      {
        other_player_action.action = EntityAction::Wait;
        other_player_action.is_ready = true;
      }
    }
  }

  for (player_pos, player_tile_parent, player_type, player_pending_action) in player_q.iter() {
    match (
      player_pending_action.is_ready,
      &player_pending_action.action,
    ) {
      (false, EntityAction::Move(_)) => {
        if map_marker_q.is_empty() {
          build_markers = Some((
            player_pos.to_owned(),
            player_tile_parent.to_owned(),
            player_type.to_owned(),
          ));
        }
      }
      _ => {}
    };
  }

  // And finally spawn new markers
  if let Some((origin, parent, player_type)) = build_markers {
    let layer = parent.layer_id;
    let map = parent.map_id;

    let max_move_distance: i32 = constants::PLAYER_DEFAULT_MOVE_SPEED as i32;

    /*
    Finding possible move tiles:
    1. Make a list of rows that are `max_move_distance` or less from the player
    2. Make a list of columns on those rows that are less than `max_move_distance - $row_distance_from_player`
    3. Turn the row, column pairs into `TilePos`'s
    4. For each TilePos, check that the tile type is correct, and the there are no entities in the way
    5. For each TilePos left over, create a MapActionChooserBundle to put a chooser component on that square
    */

    ((max_move_distance * -1 + 1)..max_move_distance)
      .map(|i| {
        let moves_left = max_move_distance - i.abs();
        let row = origin.1 as i32 + i;
        ((moves_left * -1 + 1)..(moves_left)).map(move |j| {
          let column = origin.0 as i32 + j;
          (column, row)
        })
      })
      .flatten()
      .filter_map(|(x, y)| {
        if x < 0 || y < 0 {
          None
        } else {
          Some(TilePos(x as u32, y as u32))
        }
      })
      .filter_map(move |pos| {
        if let Ok(tile_entity) = map_query.get_tile_entity(pos, map, layer) {
          if let Ok((dp, parent, data)) = tile_q.get(tile_entity) {
            assert!(*dp == pos);
            if data.kind == TileKind::Floor {
              return Some((pos, parent.to_owned()));
            }
          }
        }
        None
      })
      .for_each(|(marker_pos, marker_parent)| {
        commands
          .spawn()
          .insert_bundle(MapActionChooserBundle::new(
            &marker_pos,
            &marker_parent,
            &origin.clone(),
            &player_type,
          ))
          .insert(PlayerMoveSelect);
      });
  }
}

// If the other player has associated action choosers,
// we should despawn the old ones, or it will get confusing.
pub fn despawn_action_choosers(
  mut commands: Commands,
  players: Query<(&TilePos, &EntityPendingAction), With<PlayerStatus>>,
  choosers: Query<(Entity, &MapActionChooser)>,
) {
  for (player_pos, player_pending_action) in players.iter() {
    if player_pending_action.is_wait() || player_pending_action.is_ready {
      for (chooser_entity, chooser_origin) in choosers.iter() {
        if *player_pos == chooser_origin.origin {
          commands.entity(chooser_entity).despawn_recursive();
        }
      }
    }
  }
}

/// If a MapActionChooser component is clicked,
/// the corresponding player's EntityPendingAction should be updated to Move, with is_ready=true
pub fn select_player_move(
  mut commands: Commands,
  mut select_events: EventReader<TileSelectedEvent>,
  select_markers: Query<(&MapActionChooser, &TilePos), With<PlayerMoveSelect>>,
  player: Query<(Entity, &MapEntityType), With<PlayerStatus>>,
) {
  for TileSelectedEvent(event) in select_events.iter() {
    if let Some(clicked_tile) = event {
      for (act, marker_pos) in select_markers.iter() {
        if clicked_tile == marker_pos {
          let (player_entity, player_type) = player
            .iter()
            .find(|(_, t)| **t == MapEntityType::Player(act.player))
            .unwrap();

          if *player_type == MapEntityType::Player(act.player) {
            commands
              .entity(player_entity)
              .remove::<EntityPendingAction>();
            commands.entity(player_entity).insert(EntityPendingAction {
              action: EntityAction::Move((act.origin, clicked_tile.to_owned())),
              is_ready: true,
            });
          }
        }
      }
    }
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
      .add_system(start_turn)
      .add_system(start_player_choice)
      .add_system(run_turn_animation)
      .add_system(draw_action_arrows)
      .add_system(remove_action_arrows)
      .add_system(update_player_action)
      .add_system(create_player_move_selector.label("player-move-selector"))
      .add_system(despawn_action_choosers.after("player-move-selector"))
      .add_system(select_player_move);
  }
}
