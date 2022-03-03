use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::constants;
use crate::turn::CompletedTurn;

//use super::basic_types::{UIBlock, UIBlocks};
use super::memory_viewer::MemoryWindowState;
use super::spell_viewer::SpellViewerState;

pub fn top_bar(
  mut commands: Commands,
  mut gui: ResMut<EguiContext>,
  turns_elapsed: Query<&CompletedTurn>,
  memory_window_state: Query<(Entity, &MemoryWindowState)>,
  spell_window_state: Query<(Entity, &SpellViewerState)>,
) {
  let egui_height = gui.ctx_mut().available_rect().height() * constants::TOP_BAR_DESIRED_SIZE;

  let is_spells_shown = spell_window_state.get_single().is_ok();
  let is_memory_shown = memory_window_state.get_single().is_ok();

  let mut top_bar_background = egui::Frame::default();
  top_bar_background.fill = egui::Color32::BLACK;

  egui::TopBottomPanel::top("Top Panel")
    .min_height(constants::TOP_BAR_MIN_SIZE)
    .default_height(egui_height)
    .frame(top_bar_background)
    .show(gui.ctx_mut(), |ui| {
      ui.horizontal_top(|ui| {
        ui.add_space(10.);
        ui.heading(constants::GAME_NAME);
        ui.add_space(50.);
        ui.label(format!("turn {}", turns_elapsed.iter().count()));

        let spells_button_text = if is_spells_shown {
          "Hide Spells"
        } else {
          "Show Spells"
        };
        if ui.button(spells_button_text).clicked() {
          if !is_spells_shown {
            commands.spawn().insert(SpellViewerState::default());
          } else {
            let (entity, _) = spell_window_state.single();
            commands.entity(entity).despawn_recursive();
          }
        }

        let memory_button_text = if is_memory_shown {
          "Hide Memory"
        } else {
          "Show Memory"
        };
        if ui.button(memory_button_text).clicked() {
          if !is_memory_shown {
            commands.spawn().insert(MemoryWindowState::default());
          } else {
            let (entity, _) = memory_window_state.single();
            commands.entity(entity).despawn_recursive();
          }
        }
      });
    });
}
