use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{
  level::{AvailableLevel, CompletedLevel, CurrentLevel},
  utils, GameState, MainCamera,
};

#[derive(Component)]
pub struct MenuSprite;

/// When the player exits and level and enters the menu:
/// - need to set the camera zoom and pan back to default
/// - Draw the game name
/// - Draw the characters
pub fn menu_startup(
  mut commands: Commands,
  mut camera: Query<(&mut OrthographicProjection, &mut Transform), With<MainCamera>>,
) {
  let (mut camera_ortho, mut camera_trans) = camera.get_single_mut().unwrap();
  camera_ortho.scale = 0.5;
  camera_trans.translation = Vec3::new(0., 0., 999.);

  commands.spawn_bundle(utils::new_square_sprite_bundle(
    Color::CYAN,
    Vec2::new(50., 50.),
    Vec3::new(20., 20., 10.),
  )).insert(MenuSprite);
}

/// Undo everything we did in menu startup
pub fn menu_shutdown(mut commands: Commands, menu_decor: Query<Entity, With<MenuSprite>>) {
  for e in menu_decor.iter() {
    commands.entity(e).despawn_recursive();
  }
}

/// Draw a UI for selecting levels
/// (very quick and dirty)
pub fn quick_level_ui(
  mut commands: Commands,
  mut gui: ResMut<EguiContext>,
  mut game_state: ResMut<State<GameState>>,
  levels: Query<(Entity, &AvailableLevel, Option<&CompletedLevel>)>,
) {
  let window = gui.ctx_mut().available_rect();

  egui::Window::new("Choose a level")
    .resizable(false)
    .collapsible(false)
    .fixed_pos((0., window.height() - window.height() / 3.))
    .show(gui.ctx_mut(), |ui| {
      ui.expand_to_include_x(window.width());
      ui.expand_to_include_y(window.height());

      for (level_entity, level, is_complete) in levels.iter() {
        ui.horizontal(|ui| {
          if ui.button(level.name.as_str()).clicked() {
            game_state.set(GameState::Running).unwrap();
            commands.entity(level_entity).insert(CurrentLevel);
          }
          if is_complete.is_some() {
            ui.label("✔️");
          }
        });
      }
    });
}
