use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::{TileParent, TilePos};

use super::MainCamera;
use crate::ui::UIBlocks;
use crate::utils::initial_map_drawing_position;

/// For things that will be drawn on a top map tile
/// Probably paired with TilePos and TileParent
/// Does not exist on the actual map
#[derive(Component, Default)]
pub struct OnMap;

/// Marker component for tiles with players or enemies on them
#[derive(Component, Default)]
pub struct TileHasEntity;

#[derive(Component)]
pub struct HoveredTile;

#[derive(Component)]
pub struct SelectedTile;

#[derive(Bundle)]
pub struct HoveredTileBundle {
  tile: HoveredTile,
  map: OnMap,
  map_pos: TilePos,
  parent: TileParent,
  #[bundle]
  sprite_bundle: SpriteBundle,
}

#[derive(Bundle)]
pub struct SelectedTileBundle {
  tile: SelectedTile,
  map: OnMap,
  map_pos: TilePos,
  parent: TileParent,
  #[bundle]
  sprite_bundle: SpriteBundle,
}

/// A marking component for tiles on the IntGrid layer
/// that has tile data
#[derive(Debug, Component, Default)]
pub struct DataLayer;

#[derive(Component, Default)]
pub struct Open;

#[derive(Component, Default)]
pub struct Wall {
  health: f32,
}

#[derive(Component, Default)]
struct Floor;

#[derive(Debug, Component, Default)]
pub struct TileTemp {
  temp: f32,
}

#[derive(Bundle, LdtkIntCell)]
struct OpenTileBundle {
  l: DataLayer,
  open: Open,
  temp: TileTemp,
}

#[derive(Bundle, LdtkIntCell)]
struct WallTileBundle {
  l: DataLayer,
  wall: Wall,
  temp: TileTemp,
}

#[derive(Bundle, LdtkIntCell)]
struct FloorTileBundle {
  l: DataLayer,
  floor: Floor,
  temp: TileTemp,
}

// allows components to be drawn on the map
pub fn mapped_component(
  mut comps: Query<(&mut Transform, &TilePos, &TileParent), (With<OnMap>, Changed<TilePos>)>,
  mut map: bevy_ecs_tilemap::MapQuery,
) {
  for (mut transform, position, parent) in comps.iter_mut() {
    let (_, layer) = map.get_layer(parent.map_id, parent.layer_id).unwrap();

    let t_s_width = layer.settings.tile_size.0;
    let t_s_height = layer.settings.tile_size.1;
    let d_tx = (t_s_width / 2.) + t_s_width * position.0 as f32;
    let d_ty = (t_s_height / 2.) + t_s_height * position.1 as f32;

    transform.translation = Vec3::new(d_tx, d_ty, 5.);
  }
}

pub fn tile_mouse_hover(
  mut commands: Commands,
  windows: Res<Windows>,
  ui_blocks: Res<UIBlocks>,
  q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
  tiles: Query<(Entity, &TilePos, &TileParent), (With<DataLayer>, Without<OnMap>)>,
  mut selected: Query<(Entity, &mut TilePos, &mut TileParent), (With<OnMap>, With<HoveredTile>)>,
  mut map: bevy_ecs_tilemap::MapQuery,
) {
  let window = windows.get_primary().unwrap();
  if let Some(mouse_position) = window.cursor_position() {
    if ui_blocks.is_blocked(mouse_position) {
      return;
    }
    // Convert screen coordinates into world coordinates
    // taken from https://bevy-cheatbook.github.io/cookbook/cursor2world.html
    let (camera, camera_transform) = q_camera.single();

    // we need the window size as a Vec2
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);

    // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
    let ndc = (mouse_position / window_size) * 2.0 - Vec2::ONE;

    // matrix for undoing the projection and camera transform
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();

    // use it to convert ndc to world-space coordinates
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

    // reduce it to a 2D value
    let world_pos: Vec2 = world_pos.truncate();

    // Deal with the tiles in the Data Layer
    let mut hovered_tile = None;
    let mut iter = tiles.iter();
    while let Some((e, pos, parent)) = iter.next() {
      if let Some((_, layer)) = map.get_layer(parent.map_id, parent.layer_id) {
        let tile_size = layer.settings.tile_size;
        let tile_x_start = pos.0 as f32 * tile_size.0;
        let tile_x_end = (pos.0 + 1) as f32 * tile_size.0;
        let tile_y_start = pos.1 as f32 * tile_size.1;
        let tile_y_end = (pos.1 + 1) as f32 * tile_size.1;

        if (world_pos.x >= tile_x_start && world_pos.x < tile_x_end)
          && (world_pos.y >= tile_y_start && world_pos.y < tile_y_end)
        {
          hovered_tile = Some((pos.to_owned(), parent.to_owned()));
          commands.entity(e).insert(HoveredTile);
          break;
        }
      }
    }

    // Deal with the selection sprite on top of the data layer
    if let Some((new_pos, new_parent)) = hovered_tile {
      if let Ok((_, mut old_pos, mut old_parent)) = selected.get_single_mut() {
        if let Ok(old_entity) =
          map.get_tile_entity(old_pos.to_owned(), old_parent.map_id, old_parent.layer_id)
        {
          commands.entity(old_entity).remove::<HoveredTile>();
        }

        old_pos.0 = new_pos.0;
        old_pos.1 = new_pos.1;
        old_parent.chunk = new_parent.chunk;
      } else {
        let (_, layer) = map
          .get_layer(new_parent.map_id, new_parent.layer_id)
          .unwrap();
        commands.spawn_bundle(HoveredTileBundle {
          tile: HoveredTile,
          map: OnMap,
          map_pos: new_pos,
          parent: new_parent,
          sprite_bundle: SpriteBundle {
            sprite: Sprite {
              color: Color::rgba(0.5, 0.5, 1., 0.4),
              ..Default::default()
            },
            transform: Transform {
              scale: Vec3::new(layer.settings.tile_size.0, layer.settings.tile_size.1, 0.),
              translation: initial_map_drawing_position(&layer.settings.tile_size, &new_pos),
              ..Default::default()
            },
            ..Default::default()
          },
        });
      }
    } else {
      for (entity, pos, parent) in selected.iter() {
        commands.entity(entity).despawn_recursive();
        if let Ok(data_layer_entity) =
          map.get_tile_entity(pos.to_owned(), parent.map_id, parent.layer_id)
        {
          commands.entity(data_layer_entity).remove::<HoveredTile>();
        }
      }
    }
  }
}

pub fn select_tile(
  mut commands: Commands,
  windows: Res<Windows>,
  ui_blocks: Res<UIBlocks>,
  tile_hover_sprite: Query<
    (&TilePos, &TileParent),
    (With<HoveredTile>, With<OnMap>, Without<SelectedTile>),
  >,
  mut old_tile_select_sprite: Query<
    (Entity, &mut TilePos, &mut TileParent),
    (With<OnMap>, With<SelectedTile>),
  >,
  old_data_layer_sprite: Query<Entity, (Without<OnMap>, With<DataLayer>, With<SelectedTile>)>,
  mut events: EventReader<MouseButtonInput>,
  mut map: bevy_ecs_tilemap::MapQuery,
) {
  use bevy::input::ElementState;

  let mut did_click_happen = false;
  for event in events.iter() {
    match event.state {
      ElementState::Pressed => {
        did_click_happen = true;
      }
      _ => {}
    }
  }

  if did_click_happen {
    // double check that the mouse isn't blocked by the ui
    let window = windows.get_primary().unwrap();
    if let Some(mp) = window.cursor_position() {
      if ui_blocks.is_blocked(mp) {
        return;
      }
    }

    if let Ok((pos, parent)) = tile_hover_sprite.get_single() {
      if let Ok(data_layer_entity) =
        map.get_tile_entity(pos.to_owned(), parent.map_id, parent.layer_id)
      {
        commands.entity(data_layer_entity).insert(SelectedTile);
      }

      if let Ok((_, mut old_pos, mut old_parent)) = old_tile_select_sprite.get_single_mut() {
        // update the selected tile
        if *old_pos != *pos {
          old_pos.0 = pos.0;
          old_pos.1 = pos.1;
          old_parent.chunk = parent.chunk;

          // We also need to remove the old marker component from the data layer
          let e = old_data_layer_sprite.get_single().unwrap();
          commands.entity(e).remove::<SelectedTile>();
        }
      } else {
        // a tile has been clicked for the first time
        let (_, layer) = map.get_layer(parent.map_id, parent.layer_id).unwrap();
        commands.spawn_bundle(SelectedTileBundle {
          tile: SelectedTile,
          map: OnMap,
          map_pos: pos.to_owned(),
          parent: parent.to_owned(),
          sprite_bundle: SpriteBundle {
            sprite: Sprite {
              color: Color::rgba(0.44, 1., 0.66, 0.5),
              ..Default::default()
            },
            transform: Transform {
              scale: Vec3::new(16., 16., 0.),
              translation: initial_map_drawing_position(&layer.settings.tile_size, pos),
              ..Default::default()
            },
            ..Default::default()
          },
        });

        assert!(old_data_layer_sprite.iter().count() == 0);
        if let Ok(data_layer_entity) =
          map.get_tile_entity(pos.to_owned(), parent.map_id, parent.layer_id)
        {
          //dbg!("bye");
          commands.entity(data_layer_entity).insert(SelectedTile);
        }
      }
    }
  }
}

fn map_pan(time: Res<Time>, keys: Res<Input<KeyCode>>, mut camera: Query<&mut Transform, With<MainCamera>>, ui_block: Query<&crate::ui::BlockKeyInput>) {
  const SCROLL_FACTOR: f32 = 0.35;
  const SCROLL_LIMIT_PLUS_X: f32 = 300.;
  const SCROLL_LIMIT_MINUS_X: f32 = -200.;
  const SCROLL_LIMIT_PLUS_Y: f32 = 200.;
  const SCROLL_LIMIT_MINUS_Y: f32 = -100.;

  // check if the ui is intercepting keyboard events
  if !ui_block.is_empty() {
    return;
  }

  let mut camera_transform = camera.get_single_mut().unwrap();

  let mut scroll_x: f32 = 0.;
  let mut scroll_y: f32 = 0.;
  if keys.pressed(KeyCode::Left) || keys.pressed(KeyCode::A) {
    scroll_x = time.delta().as_millis() as f32 * SCROLL_FACTOR * -1.;
  }
  if keys.pressed(KeyCode::Right) || keys.pressed(KeyCode::D) {
    scroll_x = time.delta().as_millis() as f32 * SCROLL_FACTOR;
  }
  if keys.pressed(KeyCode::Up) || keys.pressed(KeyCode::W) {
    scroll_y = time.delta().as_millis() as f32 * SCROLL_FACTOR;
  }
  if keys.pressed(KeyCode::Down) || keys.pressed(KeyCode::S) {
    scroll_y = time.delta().as_millis() as f32 * SCROLL_FACTOR * -1.;
  }

  let new_camera_x = camera_transform.translation.x + scroll_x.round();
  if new_camera_x > SCROLL_LIMIT_MINUS_X && new_camera_x < SCROLL_LIMIT_PLUS_X {
    camera_transform.translation.x = new_camera_x;
  }
  let new_camera_y = camera_transform.translation.y + scroll_y.round();
  if new_camera_y > SCROLL_LIMIT_MINUS_Y && new_camera_y < SCROLL_LIMIT_PLUS_Y {
    camera_transform.translation.y = new_camera_y;
  }
}

fn zoom_map(mut scroll_events: EventReader<MouseWheel>, mut camera: Query<&mut OrthographicProjection, With<MainCamera>>) {
  use bevy::input::mouse::MouseScrollUnit;

  let mut change = 0.;

  for event in scroll_events.iter() {
    match event.unit {
      MouseScrollUnit::Line => {
        change += event.y;
      }
      // TODO: this event is fired when player scrolls on mouse pad.
      // need to figure out the scale of this value.
      MouseScrollUnit::Pixel => {
        change += event.y;
      }
    }
  }

  let mut ortho = camera.get_single_mut().unwrap();
  let new_scale = ortho.scale + change * 0.1;
  if new_scale > 0.1 && new_scale < 0.9 {
    ortho.scale = new_scale;
  }

  if change != 0. {
    dbg!(change);
  }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugin(LdtkPlugin)
      .insert_resource(LevelSelection::Index(0)) // Selects which ldtk level to load first
      .add_system(mapped_component)
      .add_system(tile_mouse_hover)
      .add_system(select_tile)
      .add_system(map_pan)
      .add_system(zoom_map)
      .register_ldtk_int_cell::<OpenTileBundle>(1)
      .register_ldtk_int_cell::<WallTileBundle>(2)
      .register_ldtk_int_cell::<FloorTileBundle>(3);
  }
}