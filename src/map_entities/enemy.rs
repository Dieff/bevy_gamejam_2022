use bevy::prelude::*;
use bevy_ecs_tilemap::{MapQuery, TileParent, TilePos};
use std::collections::{HashMap, HashSet};

use super::{player::PlayerStatus, EntityHealth, MapEntityType};

use crate::map::{DataLayer, DrawOnMap, TileKind};
use crate::turn::{
  EnemyTurnAnimating, EntityAction, EntityPendingAction, PendingAttack, TurnDisplayer,
};
use crate::utils;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EnemyAIType {
  StayPut,
  AttackUntilWeak,
  AttackWeakest,
  AttackClosest,
  RunAway,
}

#[derive(Component)]
pub struct Enemy {
  ai_type: EnemyAIType,
  speed: u32,
}

impl Default for Enemy {
  fn default() -> Self {
    Self {
      ai_type: EnemyAIType::AttackClosest,
      speed: 3,
    }
  }
}

#[derive(Bundle, Default)]
pub struct NewEnemyBundle {
  pub enemy: Enemy,
  pub health: EntityHealth,
  pub map_entity_type: MapEntityType,
  pub om: DrawOnMap,
  pub map_pos: TilePos,
  pub parent: TileParent,
  #[bundle]
  pub sprite: SpriteBundle,
  pub action: EntityPendingAction,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
struct HashablePos(i32, i32);

impl HashablePos {
  fn up(&self) -> Self {
    *self + HashablePos(0, 1)
  }
  fn down(&self) -> Self {
    *self + HashablePos(0, -1)
  }
  fn left(&self) -> Self {
    *self + HashablePos(1, 0)
  }
  fn right(&self) -> Self {
    *self + HashablePos(-1, 0)
  }
  fn is_on_map(&self, dim: (u32, u32)) -> bool {
    0 < self.0 && self.0 < dim.0 as i32 && 1 < self.1 && self.1 < dim.1 as i32
  }
}

impl std::ops::Add for HashablePos {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    HashablePos(self.0 + other.0, self.1 + other.1)
  }
}

impl From<&TilePos> for HashablePos {
  fn from(p: &TilePos) -> Self {
    Self(p.0 as i32, p.1 as i32)
  }
}

impl Into<TilePos> for HashablePos {
  fn into(self) -> TilePos {
    TilePos(self.0 as u32, self.1 as u32)
  }
}

#[derive(Clone, Debug)]
struct PathfindingNode {
  pos: HashablePos,
  last_pos: HashablePos,
  from_start: i32,
  to_goal: i32,
  visited: bool,
}

impl PathfindingNode {
  fn weight(&self) -> i32 {
    self.from_start + self.to_goal
  }
}

fn get_tile_distance(cur: HashablePos, goal: HashablePos) -> i32 {
  (goal.0 - cur.0).abs() + (goal.1 - cur.1).abs()
}

// Returns a list of tile positions,
// showing the path to the closest adjacent point to the goal.
fn simple_a_star(
  start: HashablePos,
  goal: HashablePos,
  blocked_tiles: HashSet<HashablePos>,
  map_dim: (u32, u32),
) -> Option<Vec<HashablePos>> {
  let mut open_tiles: HashMap<HashablePos, PathfindingNode> = HashMap::new();
  open_tiles.insert(
    start,
    PathfindingNode {
      pos: start,
      last_pos: start,
      from_start: 0,
      to_goal: get_tile_distance(start, goal),
      visited: true,
    },
  );

  let mut cur = start;

  'outer: loop {
    let new_positions: Vec<PathfindingNode> = [cur.up(), cur.down(), cur.left(), cur.right()]
      .iter()
      .filter(|pos| pos.is_on_map(map_dim))
      .filter(|pos| !blocked_tiles.contains(pos))
      .map(|pos| PathfindingNode {
        pos: pos.to_owned(),
        last_pos: cur.clone(),
        from_start: get_tile_distance(*pos, start),
        to_goal: get_tile_distance(*pos, goal),
        visited: false,
      })
      .collect();

    for mut node in new_positions.into_iter() {
      let node_pos = node.pos;
      if node.to_goal == 1 {
        open_tiles.insert(node_pos, node);
        cur = node_pos;
        break 'outer;
      }
      if let Some(old_node) = open_tiles.get(&node_pos) {
        if node.weight() < old_node.weight()
          || node.weight() == old_node.weight() && node.to_goal < old_node.to_goal
        {
          if old_node.visited {
            node.visited = true;
          }
          open_tiles.insert(node_pos, node);
        }
      } else {
        open_tiles.insert(node_pos.clone(), node);
      }
    }

    let mut best: Option<PathfindingNode> = None;
    for (_, node) in open_tiles.iter() {
      if !node.visited {
        if let Some(cur_best) = best.as_ref() {
          if node.weight() < cur_best.weight() {
            best = Some(node.clone());
          }
        } else {
          best = Some(node.clone());
        }
      }
    }

    if let Some(new_cur) = best {
      cur = new_cur.pos;
      open_tiles.get_mut(&cur).unwrap().visited = true;
    } else {
      // There are no spaces that haven't been visited
      // so no path can be found.
      return None;
    }
  }

  let mut steps: Vec<HashablePos> = vec![];
  let mut last = cur;
  while last != start {
    let previous_node = open_tiles.get(&last).unwrap();
    steps.push(previous_node.pos);
    last = previous_node.last_pos;
  }
  steps.reverse();
  Some(steps)
}

fn get_move(
  start: &TilePos,
  goal: &TilePos,
  speed: u32,
  blocked: &Vec<TilePos>,
  map_dim: (u32, u32),
) -> Option<(TilePos, bool)> {
  if start == goal {
    return None;
  }

  if utils::tile_distance(start, goal) == 1 {
    return Some((start.to_owned(), true));
  }

  let mut blocked_set = HashSet::new();
  blocked.into_iter().map(HashablePos::from).for_each(|p| {
    blocked_set.insert(p);
  });

  simple_a_star(start.into(), goal.into(), blocked_set, map_dim)
    .map(|mut path| {
      if speed > path.len() as u32 {
        path.pop().unwrap().into()
      } else {
        path[((speed - 1) as usize)].into()
      }
    })
    .map(|pos| {
      (pos, utils::tile_distance(&pos, goal) == 1)
    })
}

fn run_in_direction(
  pos: &TilePos,
  speed: u32,
  map_dim: (u32, u32),
  direction: (i32, i32),
  blocked: &Vec<TilePos>,
) -> EntityAction {
  let i_map_dim = (map_dim.0 as i32, map_dim.1 as i32);
  let mut new_x = pos.0 as i32;
  let mut new_y = pos.1 as i32;
  for _ in 1..speed {
    let test_x = new_x + direction.0 * 1;
    let test_y = new_y + direction.1 * 1;

    if test_x < 0 || test_x > i_map_dim.0 - 1 {
      break;
    }
    if test_y < 0 || test_y > i_map_dim.1 - 1 {
      break;
    }

    if blocked
      .iter()
      .find(|p| p.0 as i32 == test_x && p.1 as i32 == test_y)
      .is_some()
    {
      break;
    }

    new_x = test_x;
    new_y = test_y;
  }
  let new_pos = TilePos(new_x as u32, new_y as u32);
  if utils::tile_distance(pos, &new_pos) == 0 {
    EntityAction::Wait
  } else {
    EntityAction::Move((pos.to_owned(), new_pos))
  }
}

/// Creates the correct actions for enemies after the player turn has ended
pub fn enemy_ai(
  run_ai: Query<&TurnDisplayer, Added<EnemyTurnAnimating>>,
  mut enemies: Query<(
    &mut EntityPendingAction,
    &Enemy,
    &EntityHealth,
    &TilePos,
    &TileParent,
  )>,
  players: Query<(Entity, &TilePos, &PlayerStatus)>,
  tiles: Query<(&TilePos, &DataLayer)>,
  mut map_q: MapQuery,
) {
  // An enemy turn has started when there is both a TurnDisplayer and a EnemyTurnAnimating component added
  if let Err(_) = run_ai.get_single() {
    return;
  }
  // ok, the enemy turn has started

  // make a list of all the blocked points to use in pathfinding. This seems inefficient,
  // but the borrows on MapQuery and Query<&TilePos> are really hard to figure out.
  let mut blocked_points: Vec<TilePos> = tiles
    .iter()
    .filter_map(|(tp, dl)| match dl.kind {
      TileKind::Floor => None,
      _ => Some(tp.to_owned()),
    })
    .collect();

  let mut warrior_pos: (Entity, TilePos) = (Entity::from_raw(0), Default::default());
  let mut wizard_pos: (Entity, TilePos) = (Entity::from_raw(0), Default::default());
  for p in players.iter() {
    blocked_points.push(p.1.to_owned());
    if p.2.magika.is_some() {
      wizard_pos = (p.0, p.1.to_owned());
    } else {
      warrior_pos = (p.0, p.1.to_owned());
    }
  }

  for (mut queued_action, enemy_data, enemy_health, enemy_pos, parent) in enemies.iter_mut() {
    let layer = map_q.get_layer(parent.map_id, parent.layer_id).unwrap().1;
    let ms = layer.get_layer_size_in_tiles();
    let map_dim = (ms.0, ms.1);

    let mut ai_type = enemy_data.ai_type;
    if ai_type == EnemyAIType::AttackUntilWeak {
      if enemy_health.health < 20. {
        ai_type = EnemyAIType::RunAway;
      } else {
        ai_type = EnemyAIType::AttackClosest;
      }
    }

    let closest_player = if utils::tile_distance(enemy_pos, &warrior_pos.1)
      > utils::tile_distance(enemy_pos, &wizard_pos.1)
    {
      wizard_pos
    } else {
      warrior_pos
    };

    queued_action.is_ready = true;

    // If an enemy is next to a player, the enemy will always attack
    if utils::tile_distance(&closest_player.1, enemy_pos) == 1 {
      queued_action.action = EntityAction::Attack(PendingAttack::new(
        closest_player.0,
        closest_player.1.to_owned(),
        enemy_pos.to_owned(),
        enemy_pos.to_owned(),
      ));
    } else {
      match ai_type {
        EnemyAIType::AttackClosest => {
          if let Some((new_pos, reached_target)) = get_move(
            enemy_pos,
            &closest_player.1,
            enemy_data.speed,
            &blocked_points,
            map_dim,
          ) {
            blocked_points.push(new_pos);
            if reached_target {
              queued_action.action = EntityAction::Attack(PendingAttack::new(
                closest_player.0,
                closest_player.1.clone(),
                new_pos,
                enemy_pos.to_owned(),
              ));
            } else {
              queued_action.action = EntityAction::Move((enemy_pos.to_owned(), new_pos));
            }
          } else {
            // a path was not found, so just wait
            queued_action.action = EntityAction::Wait;
          }
        }
        EnemyAIType::RunAway => {
          let warrior_pos = warrior_pos.1;
          // just run away from warrior for now
          if warrior_pos.1 > enemy_pos.1 {
            queued_action.action = run_in_direction(
              enemy_pos,
              enemy_data.speed,
              map_dim,
              (0, -1),
              &blocked_points,
            );
          } else if warrior_pos.1 < enemy_pos.1 {
            queued_action.action = run_in_direction(
              enemy_pos,
              enemy_data.speed,
              map_dim,
              (0, 1),
              &blocked_points,
            );
          } else if warrior_pos.0 > enemy_pos.0 {
            queued_action.action = run_in_direction(
              enemy_pos,
              enemy_data.speed,
              map_dim,
              (-1, 0),
              &blocked_points,
            );
          } else if warrior_pos.0 < enemy_pos.0 {
            queued_action.action = run_in_direction(
              enemy_pos,
              enemy_data.speed,
              map_dim,
              (1, 0),
              &blocked_points,
            );
          } else {
            queued_action.action = EntityAction::Wait;
          }
        }

        _ => {
          queued_action.action = EntityAction::Wait;
        }
      }
    }

    if !(queued_action.is_move() || queued_action.is_attack()) {
      blocked_points.push(enemy_pos.to_owned());
    }
  }
}
