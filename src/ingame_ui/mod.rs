use bevy::prelude::*;
use bevy_egui::EguiContext;

use crate::GameState;

pub mod basic_types;
pub use basic_types::BlockKeyInput;

use self::{memory_viewer::MemoryWindowState, spell_viewer::SpellViewerState};
mod entity_viewer;
mod memory_viewer;
mod round_summary;
mod sides;
mod spell_viewer;
mod top_bar;

/// Give the UI systems their own stage so that we can be sure they have the correct order
static UI_STAGE: &str = "UISTAGE";

/// Stores whether or not the mouse was over a UI element on the last frame.
#[derive(Default)]
pub struct PointerStatus(bool);

impl PointerStatus {
  /// If true, disregard mouse events over the map
  pub fn disregard_mouse_event(&self) -> bool {
    self.0
  }
}

/// updates `Res<PointerStatus>` to make sure that mouse events are intercepted to egui, and not used in the map
fn ui_block_check(mut gui_ctx: ResMut<EguiContext>, mut p_status: ResMut<PointerStatus>) {
  p_status.0 = gui_ctx.ctx_mut().is_pointer_over_area();
}

/// removes a few misc components when we transition from game to menu
fn game_exit(
  mut commands: Commands,
  old_entities: Query<Entity, Or<(With<MemoryWindowState>, With<SpellViewerState>)>>,
) {
  for e in old_entities.iter() {
    commands.entity(e).despawn_recursive();
  }
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugin(bevy_egui::EguiPlugin)
      .insert_resource(PointerStatus::default())
      // It is important that each egui UI piece is created in the same order every frame, so we make a new single-threaded stage
      // for simplicity. Also need to propogate the state to it.
      .add_stage_after(CoreStage::Update, UI_STAGE, SystemStage::single_threaded())
      .add_state_to_stage(UI_STAGE, GameState::default())
      .add_system_set_to_stage(
        UI_STAGE,
        SystemSet::on_update(GameState::Running)
          .with_system(top_bar::top_bar.label("top-bar"))
          .with_system(sides::left_panel.label("left-bar").after("top-bar"))
          .with_system(memory_viewer::memory_ui.after("top-bar").before("left-bar"))
          .with_system(
            spell_viewer::spell_viewer
              .after("top-bar")
              .before("left-bar"),
          )
          .with_system(round_summary::round_summary.before("top-bar")),
      )
      .add_system_set(SystemSet::on_exit(GameState::Running).with_system(game_exit))
      .add_system_to_stage(CoreStage::Last, ui_block_check);
  }
}
