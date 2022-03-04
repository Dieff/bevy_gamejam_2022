use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::level::{RoundSummary, ToMenu};
use crate::turn::CompletedTurn;

pub fn round_summary(
  mut gui: ResMut<EguiContext>,
  mut quit_event: EventWriter<ToMenu>,
  round_over_q: Query<&RoundSummary>,
  completed_turns: Query<&CompletedTurn>,
) {
  let window = gui.ctx_mut().available_rect();
  let left_margin = window.width() / 20.;
  let top_margin = window.height() / 20.;
  let right = window.width() - (window.width() / 20.);
  let bottom = window.height() - (window.height() / 20.);

  if let Ok(round_summary) = round_over_q.get_single() {
    let result_text = match round_summary.0 {
      crate::level::RoundResult::Victory => "Victory",
      crate::level::RoundResult::Defeat => "Defeat",
      crate::level::RoundResult::Neutral => "No Contest",
    };

    egui::Window::new(result_text)
      .resizable(false)
      .collapsible(false)
      .fixed_pos((left_margin, top_margin))
      .show(gui.ctx_mut(), |ui| {
        ui.label(format!(
          "You fought for {} turns.",
          completed_turns.iter().count()
        ));

        if ui.button("Back To Menu").clicked() {
          quit_event.send(ToMenu);
        }
        ui.expand_to_include_x(right);
        ui.expand_to_include_y(bottom);
      });
  }
}
