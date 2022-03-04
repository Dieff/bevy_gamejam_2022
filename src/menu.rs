use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{
  level::{AvailableLevel, CompletedLevel, CurrentLevel},
  GameState, MainCamera,
};

#[derive(Component)]
pub struct MenuSprite;

/// When the player exits and level and enters the menu:
/// - need to set the camera zoom and pan back to default
/// - Draw the game name
/// - Draw the characters
pub fn menu_startup(
  mut commands: Commands,
  assets: Res<AssetServer>,
  windows: Res<Windows>,
  mut camera: Query<(&mut OrthographicProjection, &mut Transform), With<MainCamera>>,
) {
  let (mut camera_ortho, mut camera_trans) = camera.get_single_mut().unwrap();
  camera_ortho.scale = 0.5;
  camera_trans.translation = Vec3::new(0., 0., 999.);

  let primary_window = windows.get_primary().unwrap();

  commands
    .spawn_bundle(SpriteBundle {
      texture: assets.load("swpt_title.png"),
      sprite: Sprite {
        // Since it is a background, we should fit the sprite to the window
        custom_size: Some(Vec2::new(
          primary_window.width() / 2.,
          primary_window.height() / 2.,
        )),
        ..Default::default()
      },
      ..Default::default()
    })
    .insert(MenuSprite);
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

      ui.horizontal(|ui| {
        ui.label("Made with");
        ui.hyperlink_to("Bevy", "https://bevyengine.org/");
        ui.label("for the first official");
        ui.hyperlink_to("Bevy game jam.", "https://itch.io/jam/bevy-jam-1/entries");
        ui.label("Warning: game is very WIP.")
      });

      ui.add_space(10.);
      for (level_entity, level, is_complete) in levels.iter() {
        ui.horizontal(|ui| {
          if ui.button(level.name.as_str()).clicked() {
            game_state.set(GameState::Running).unwrap();
            commands.entity(level_entity).insert(CurrentLevel);
          }
          if is_complete.is_some() {
            // I love using emojis in my code!
            ui.label("✔️");
          }
        });
      }

    });
}
