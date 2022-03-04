use std::cmp::Ordering;

/// Handles components that allow a player to select actions on map,
/// such as where to move or which enemy to attack.
use bevy::prelude::*;
use bevy_ecs_tilemap::{MapQuery, TileParent, TilePos, TileSize};
use bevy_prototype_lyon::prelude::*;

use crate::constants;
use crate::map::{DataLayer, DrawOnMap, SelectedTile, TileKind, TileSelectedEvent};
use crate::map_entities::enemy::Enemy;
use crate::map_entities::{player::PlayerStatus, MapEntityType, PlayerType};
use crate::utils;

use super::{EntityAction, EntityPendingAction, PendingAttack, TurnDisplayer};

#[derive(Component, Debug)]
pub struct ActionIndicationMarker;

/// Inserts action arrows for pending player actions.
/// Only add the arrows when the action has been fully chosen `(EntityPendingAction::is_ready == true)`
pub fn draw_action_arrows(
  mut commands: Commands,
  pending: Query<
    (&TilePos, &EntityPendingAction),
    (Changed<EntityPendingAction>, With<PlayerStatus>),
  >,
  active_turn: Query<&TurnDisplayer>,
) {
  // Don't want to draw arrows while a turn is being animated
  if !active_turn.is_empty() {
    return;
  }

  pending
    .iter()
    .filter_map(
      |(center, action)| match (action.is_ready, action.action.to_owned()) {
        (true, EntityAction::Move((a, b))) => Some((true, a, b)),
        (
          true,
          EntityAction::Attack(PendingAttack {
            enemy_position: target,
            ..
          }),
        ) => Some((false, center.to_owned(), target)),
        _ => None,
      },
    )
    .for_each(|(is_moving, start, end)| {
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

      // We create an arrow by finding the angle of the action vector, and the drawing to smaller lines
      // with slightly offset angles from the end of the vector.
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
      // TODO: surely I could simplify this with better math
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
                if is_moving {
                  constants::MOVE_INDICATION_ARROW_COLOR
                } else {
                  constants::ATTACK_INDICATION_ARROW_COLOR
                },
                3.,
              )),
              transform,
            ),
        )
        .insert(ActionIndicationMarker)
        .insert(start.clone());
    });
}

/// Remove the action indicator arrows when they are no longer needed:
/// the player is choosing a different action, or the turn is animating
pub fn remove_action_arrows(
  mut commands: Commands,
  player_q: Query<(&TilePos, &EntityPendingAction), With<PlayerStatus>>,
  arrow_q: Query<(Entity, &TilePos), With<ActionIndicationMarker>>,
  turn_q: Query<&TurnDisplayer>,
) {
  // If a turn is animating, despawn the arrows
  if !turn_q.is_empty() {
    for (e, _) in arrow_q.iter() {
      commands.entity(e).despawn_recursive();
    }
    return;
  }
  for (player_pos, player_pending_action) in player_q.iter() {
    if !(player_pending_action.is_move() || player_pending_action.is_attack())
      || !player_pending_action.is_ready
    {
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

#[derive(Component, Default)]
pub struct PlayerMoveSelect;

#[derive(Component)]
pub struct PlayerAttackSelect {
  enemy_entity: Entity,
  enemy_pos: TilePos,
}

// be sure to insert the correct marker component as well
#[derive(Bundle)]
struct MapActionChooserBundle {
  p: MapActionChooser,
  #[bundle]
  sprite: SpriteBundle,
  pos: TilePos,
  parent: TileParent,
  onmap: DrawOnMap,
}

#[derive(Bundle)]
struct PlayerMoveChooserBundle {
  #[bundle]
  macb: MapActionChooserBundle,
  kind: PlayerMoveSelect,
}

#[derive(Bundle)]
struct PlayerAttackChooserBundle {
  #[bundle]
  macb: MapActionChooserBundle,
  kind: PlayerAttackSelect,
}

impl MapActionChooserBundle {
  fn new(
    pos: &TilePos,
    parent: &TileParent,
    player_pos: &TilePos,
    player_type: &MapEntityType,
    color: Color,
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
        color,
        Vec2::new(16., 16.),
        crate::utils::initial_map_drawing_position(
          &TileSize(constants::TILE_SIZE, constants::TILE_SIZE),
          pos,
          constants::MAP_UI_Z_LEVEL,
        ),
      ),
    }
  }
}

impl PlayerMoveChooserBundle {
  fn new(
    pos: &TilePos,
    parent: &TileParent,
    player_pos: &TilePos,
    player_type: &MapEntityType,
  ) -> Self {
    Self {
      macb: MapActionChooserBundle::new(
        pos,
        parent,
        player_pos,
        player_type,
        constants::MOVE_SELECTOR_COLOR,
      ),
      kind: PlayerMoveSelect,
    }
  }
}

impl PlayerAttackChooserBundle {
  fn new(
    pos: &TilePos,
    parent: &TileParent,
    player_pos: &TilePos,
    player_type: &MapEntityType,
    enemy_pos: &TilePos,
    enemy_entity: Entity,
  ) -> Self {
    Self {
      macb: MapActionChooserBundle::new(
        pos,
        parent,
        player_pos,
        player_type,
        constants::ATTACK_SELECTOR_COLOR,
      ),
      kind: PlayerAttackSelect {
        enemy_entity,
        enemy_pos: enemy_pos.to_owned(),
      },
    }
  }
}

/// If a player has been clicked during the PlayerChoose phase of a turn,
/// or "Move" has been selected in the left UI,
/// this will spawn markers to show possible moves.
pub fn spawn_action_choosers(
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
  enemy_q: Query<(Entity, &TilePos, &TileParent), (With<Enemy>, Without<PlayerStatus>)>,
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

  // Or, move has been selected in the UI
  for (player_pos, player_tile_parent, player_type, player_pending_action) in player_q.iter() {
    match (
      player_pending_action.is_ready,
      &player_pending_action.action,
    ) {
      (false, EntityAction::Move(_)) | (false, EntityAction::Attack(_)) => {
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
        // Check if a player is blocking the move
        if player_q.iter().any(|pd| {
          match pd.3.action {
            EntityAction::Move((_, b)) => {
              if b == pos {
                return true;
              }
            }
            _ => {}
          };
          *pd.0 == pos
        }) {
          return None;
        }

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
        if let Some(enemy) = enemy_q.iter().find(|(_, pos, _)| **pos == marker_pos) {
          commands
            .spawn()
            .insert_bundle(PlayerAttackChooserBundle::new(
              &marker_pos,
              &marker_parent,
              &origin.clone(),
              &player_type,
              enemy.1,
              enemy.0,
            ));
        } else {
          commands.spawn().insert_bundle(PlayerMoveChooserBundle::new(
            &marker_pos,
            &marker_parent,
            &origin.clone(),
            &player_type,
          ));
        }
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
  select_events: Query<&TilePos, (Added<SelectedTile>, Changed<SelectedTile>)>,
  select_markers: Query<(&MapActionChooser, &TilePos), With<PlayerMoveSelect>>,
  player: Query<(Entity, &MapEntityType), With<PlayerStatus>>,
) {
  for clicked_tile in select_events.iter() {
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

pub fn select_player_attack(
  mut commands: Commands,
  select_events: Query<&TilePos, (Added<SelectedTile>, Changed<SelectedTile>)>,
  select_markers: Query<(&MapActionChooser, &TilePos, &PlayerAttackSelect)>,
  player: Query<(Entity, &MapEntityType, &TileParent, &TilePos), With<PlayerStatus>>,
) {
  for tile_pos in select_events.iter() {
    for (action_chooser, marker_pos, pas) in select_markers.iter() {
      if tile_pos == marker_pos {
        if let Some(attacking_player) = player
          .iter()
          .find(|(_, pt, ..)| **pt == MapEntityType::Player(action_chooser.player))
        {
          commands
            .entity(attacking_player.0)
            .remove::<EntityPendingAction>();

          let st = [
            utils::adjust_tile_pos(tile_pos, (1, 0)),
            utils::adjust_tile_pos(tile_pos, (-1, 0)),
            utils::adjust_tile_pos(tile_pos, (0, 1)),
            utils::adjust_tile_pos(tile_pos, (0, -1)),
          ]
          .iter()
          .map(|p| (p, utils::euclidean_tile_distance(p, &action_chooser.origin)))
          .min_by(|a, b| {
            if a.1 < b.1 {
              Ordering::Less
            } else if a.1 > b.1 {
              Ordering::Greater
            } else {
              Ordering::Equal
            }
          })
          .map(|(p, _)| p)
          .unwrap_or(&utils::adjust_tile_pos(marker_pos, (1, 0)))
          .to_owned();

          //let map_starting_tile = map_q.get_tile_entity(*tile_pos, map_id, layer_id)

          commands
            .entity(attacking_player.0)
            .insert(EntityPendingAction {
              action: EntityAction::Attack(PendingAttack {
                enemy_entity: pas.enemy_entity,
                enemy_position: pas.enemy_pos,
                new_standing_position: st.to_owned(),
                starting_position: attacking_player.3.to_owned(),
              }),
              is_ready: true,
            });
        }
      }
    }
  }
}

pub fn despawn_ui_elements(
  mut commands: Commands,
  elements: Query<Entity, Or<(With<MapActionChooser>, With<ActionIndicationMarker>)>>,
) {
  for e in elements.iter() {
    commands.entity(e).despawn_recursive();
  }
}
