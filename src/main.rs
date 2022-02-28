use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_egui::EguiPlugin;
use ui::UIBlocks;

mod map;
mod ui;
mod utils;
mod player;
mod turn;
mod wizard_lang;
mod wizard_memory;
mod wizard_types;


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

  let view: ui::MemoryView = Default::default();
  commands.spawn().insert(view);
}


fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(EguiPlugin)
    .add_plugin(map::MapPlugin)
    .add_event::<turn::StartPlayerTurn>()
    .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4))) // background color
    .insert_resource(wizard_memory::MemoryBlob::new())
    .insert_resource(UIBlocks::default())
    .register_ldtk_entity::<player::PlayerStartBundle>("Player1Start")
    .register_ldtk_entity::<player::PlayerStartBundle>("Player2Start")
    .add_startup_system(startup)
    .add_system(turn::do_turn)
    .add_system(ui::memory_ui)
    .add_system(ui::left_panel)
    .add_system(player::spawn_players_on_map)
    .run();
}
