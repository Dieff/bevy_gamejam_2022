use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::{TileParent, TilePos};
use bevy_egui::EguiPlugin;
use utils::initial_map_drawing_position;

mod ui;
mod utils;
mod wizard_lang;
mod wizard_memory;
mod wizard_types;

/// Used when we need to execute a new turn
pub struct TurnEvent;

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

/// For things that will be drawn on a map tile
#[derive(Component)]
struct OnMap;

#[derive(Component)]
struct HoveredTile;

#[derive(Component)]
struct SelectedTile;

#[derive(Bundle)]
struct HoveredTileBundle {
  tile: HoveredTile,
  map: OnMap,
  map_pos: TilePos,
  parent: TileParent,
  #[bundle]
  sprite_bundle: SpriteBundle,
}

#[derive(Bundle)]
struct SelectedTileBundle {
  tile: SelectedTile,
  map: OnMap,
  map_pos: TilePos,
  parent: TileParent,
  #[bundle]
  sprite_bundle: SpriteBundle,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
  let mut camera = OrthographicCameraBundle::new_2d();
  camera.transform = camera
    .transform
    .with_translation(Vec3::new(150., 100., 999.));
  camera.orthographic_projection.scale = 0.5;
  commands.spawn_bundle(camera).insert(MainCamera);

  commands.spawn_bundle(LdtkWorldBundle {
    ldtk_handle: asset_server.load("tiles/test2.ldtk"),
    ..Default::default()
  });

  let view: ui::MemoryView = Default::default();
  commands.spawn().insert(view);
}

fn do_turn(mut event: EventReader<TurnEvent>, mut memory: ResMut<wizard_memory::MemoryBlob>) {
  if event.iter().count() > 0 {
    // run the turn
    memory.reset_player_memory();
  }
}

// allows components to be drawn on the map
fn mapped_component(
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

fn tile_mouse_hover(
  mut commands: Commands,
  windows: Res<Windows>,
  q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
  tiles: Query<(&TilePos, &TileParent), Without<HoveredTile>>,
  mut map: bevy_ecs_tilemap::MapQuery,
  mut selected: Query<(Entity, &mut TilePos), With<HoveredTile>>,
) {
  let window = windows.get_primary().unwrap();
  if let Some(mouse_position) = window.cursor_position() {
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

    let mut tpos = None;
    for (pos, parent) in tiles.iter() {
      if let Some((_, layer)) = map.get_layer(parent.map_id, parent.layer_id) {
        let tile_size = layer.settings.tile_size;
        let tile_x_start = pos.0 as f32 * tile_size.0;
        let tile_x_end = (pos.0 + 1) as f32 * tile_size.0;
        let tile_y_start = pos.1 as f32 * tile_size.1;
        let tile_y_end = (pos.1 + 1) as f32 * tile_size.1;

        if (world_pos.x >= tile_x_start && world_pos.x < tile_x_end)
          && (world_pos.y >= tile_y_start && world_pos.y < tile_y_end)
        {
          tpos = Some((pos.to_owned(), parent.to_owned()));
        }
      }
    }

    if let Some((ntpos, parent)) = tpos {
      if let Ok((_, mut stpos)) = selected.get_single_mut() {
        stpos.0 = ntpos.0;
        stpos.1 = ntpos.1;
      } else {
        // We need to calculate the translation to draw the new sprite on top of the current tile.
        // The mapped_component system will do that as well, but not until the next frame
        let (_, layer) = map.get_layer(parent.map_id, parent.layer_id).unwrap();

        commands.spawn_bundle(HoveredTileBundle {
          tile: HoveredTile,
          map: OnMap,
          map_pos: ntpos,
          parent,
          sprite_bundle: SpriteBundle {
            sprite: Sprite {
              color: Color::rgba(0.5, 0.5, 1., 0.4),
              ..Default::default()
            },
            transform: Transform {
              scale: Vec3::new(layer.settings.tile_size.0, layer.settings.tile_size.1, 0.),
              translation: initial_map_drawing_position(&layer.settings.tile_size, &ntpos),
              ..Default::default()
            },
            ..Default::default()
          },
        });
      }
    } else {
      if let Ok((e, _)) = selected.get_single_mut() {
        commands.entity(e).despawn_recursive();
      }
    }
  }
}

fn select_tile(
  mut commands: Commands,
  tile_q: Query<(&TilePos, &TileParent), (With<HoveredTile>, Without<SelectedTile>)>,
  mut select_q: Query<(Entity, &mut TilePos), With<SelectedTile>>,
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
    if let Ok((pos, parent)) = tile_q.get_single() {
      if let Ok((_, mut old_pos)) = select_q.get_single_mut() {
        old_pos.0 = pos.0;
        old_pos.1 = pos.1;
      } else {
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
      }
    }
  }
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(LdtkPlugin)
    .add_plugin(EguiPlugin)
    //.add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default()) // Debug
    //.add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default()) // Debug
    .add_event::<TurnEvent>()
    .insert_resource(wizard_memory::MemoryBlob::new())
    .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4))) // background color
    .insert_resource(LevelSelection::Index(0)) // Selects which ldtk level to load first
    .add_startup_system(startup)
    .add_system(do_turn)
    .add_system(ui::memory_ui)
    .add_system(mapped_component)
    .add_system(tile_mouse_hover)
    .add_system(select_tile)
    .run();
}
