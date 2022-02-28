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

pub fn initial_map_drawing_position(tile_size: &TileSize, tile_pos: &TilePos) -> Vec3 {
  let thx = tile_size.0;
  let thy = tile_size.1;
  Vec3::new(
    (thx / 2.) + (tile_pos.0 as f32 * thx),
    (thy / 2.) + (tile_pos.1 as f32 * thy),
    10.,
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
