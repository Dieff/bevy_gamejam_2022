use bevy::prelude::*;
use bevy_egui::{egui, egui::Vec2 as EGVec2, EguiContext};

use crate::spells::AvailableSpell;

#[derive(Component, Default)]
pub struct SpellViewerState {
  selected_spell: Option<String>,
}

pub fn spell_viewer(
  mut gui_ctx: ResMut<EguiContext>,
  mut state_q: Query<&mut SpellViewerState>,
  spells: Query<&AvailableSpell>,
) {
  let state = state_q.get_single_mut();
  if state.is_err() {
    return;
  }
  let mut state = state.unwrap();

  egui::Window::new("Spells").show(gui_ctx.ctx_mut(), |ui| {
    ui.horizontal(|ui| {
      let mut available_width = 500.;
      ui.vertical(|ui| {
        //ui.allocate_space(EGVec2::new(available_width / 3., 1.));
        ui.allocate_at_least(EGVec2::new(10., 1.), egui::Sense::click());
        available_width = available_width - (available_width / 3.);
        ui.heading("Spells");
        ui.vertical(|ui| {
          for spell in spells.iter() {
            if ui.button(&spell.name).clicked() {
              state.selected_spell = Some(spell.name.clone());
            }
          }
        });
      });
      ui.group(|ui| {
        ui.allocate_space(EGVec2::new(available_width, 1.));
        ui.label("Editor");
      });

    });

  });
}
