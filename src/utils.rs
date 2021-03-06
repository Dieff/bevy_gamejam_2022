use bevy::prelude::*;
use bevy_ecs_tilemap::{TilePos, TileSize};

pub fn pad_string_right(victim: &mut String, count: usize) {
  while victim.len() < count {
    victim.push(' ');
  }
}

pub fn pad_string_left(victim: &mut String, count: usize) {
  let mut blank = String::with_capacity(count - victim.len());
  while blank.len() < blank.capacity() {
    blank.push(' ');
  }
  victim.insert_str(0, blank.as_str());
}

pub fn initial_map_drawing_position(
  tile_size: &TileSize,
  tile_pos: &TilePos,
  z_level: f32,
) -> Vec3 {
  let thx = tile_size.0;
  let thy = tile_size.1;
  Vec3::new(
    (thx / 2.) + (tile_pos.0 as f32 * thx),
    (thy / 2.) + (tile_pos.1 as f32 * thy),
    z_level,
  )
}

pub fn world_pos_from_tile_pos(tile_pos: &TilePos, tile_size: &TileSize) -> Vec2 {
  Vec2::new(
    tile_pos.0 as f32 * tile_size.0 + tile_size.0 / 2.,
    tile_pos.1 as f32 * tile_size.1 + tile_size.1 / 2.,
  )
}

pub fn new_square_sprite_bundle(color: Color, size: Vec2, pos: Vec3) -> SpriteBundle {
  SpriteBundle {
    sprite: Sprite {
      color,
      ..Default::default()
    },
    transform: Transform {
      scale: Vec3::new(size.x, size.y, 0.),
      translation: pos,
      ..Default::default()
    },
    ..Default::default()
  }
}

pub fn adjust_tile_pos(pos: &TilePos, adjustment: (i32, i32)) -> TilePos {
  TilePos(
    (pos.0 as i32 - adjustment.0).try_into().unwrap_or(0),
    (pos.1 as i32 - adjustment.1).try_into().unwrap_or(0),
  )
}

/// Since entities can't move diagonally, just add the dx and dy
pub fn tile_distance(a: &TilePos, b: &TilePos) -> u32 {
  ((b.0 as i32 - a.0 as i32).abs() + (b.1 as i32 - a.1 as i32)).abs() as u32
}

/// Used to draw arrows
pub fn euclidean_tile_distance(a: &TilePos, b: &TilePos) -> f32 {
  let dx = a.0.abs_diff(b.0);
  let dy = a.1.abs_diff(b.1);
  (dx.pow(2) as f32 + dy.pow(2) as f32).sqrt()
}
