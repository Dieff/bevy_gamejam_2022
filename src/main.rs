use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_prototype_lyon::prelude::*;

// for debugging
//use bevy_inspector_egui::WorldInspectorPlugin;

mod constants;
mod ingame_ui;
mod level;
mod map;
mod map_entities;
mod menu;
mod spells;
mod turn;
mod utils;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
  Menu,
  Running,
}

impl Default for GameState {
  fn default() -> Self {
    Self::Menu
  }
}

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

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
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(ShapePlugin)
    //.add_plugin(WorldInspectorPlugin::new())
    .add_plugin(ingame_ui::UIPlugin)
    .add_plugin(map::MapPlugin)
    .add_plugin(map_entities::MapEntityPlugin)
    .add_plugin(turn::TurnPlugin)
    .add_plugin(spells::SpellsPlugin)
    .add_plugin(level::LevelPlugin)
    .add_state(GameState::default())
    .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(menu::menu_startup))
    .add_system_set(SystemSet::on_update(GameState::Menu).with_system(menu::quick_level_ui))
    .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(menu::menu_shutdown))
    .insert_resource(WindowDescriptor {
      title: constants::GAME_NAME.to_string(),
      ..Default::default()
    })
    .insert_resource(ClearColor(Color::rgb(0., 0., 0.))) // background color
    .add_startup_system(startup)
    .run();
}
