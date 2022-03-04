use bevy::prelude::*;
use bevy_egui::EguiContext;

pub mod basic_types;
pub use basic_types::{BlockKeyInput};
mod sides;
mod top_bar;
mod spell_viewer;
mod memory_viewer;
mod entity_viewer;
mod round_summary;

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

/// updates `Res<PointerStatus>`
fn ui_block_check(mut gui_ctx: ResMut<EguiContext>, mut p_status: ResMut<PointerStatus>) {
  p_status.0 = gui_ctx.ctx_mut().is_pointer_over_area();
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugin(bevy_egui::EguiPlugin)
      .insert_resource(PointerStatus::default())
      // It is important that each egui UI piece is created in the same order every frame
      .add_stage_after(CoreStage::Update, UI_STAGE, SystemStage::single_threaded())
      .add_system_to_stage(UI_STAGE, top_bar::top_bar.label("top-bar"))
      .add_system_to_stage(UI_STAGE, sides::left_panel.label("left-bar").after("top-bar"))
      .add_system_to_stage(UI_STAGE, memory_viewer::memory_ui.after("top-bar"))
      .add_system_to_stage(UI_STAGE, spell_viewer::spell_viewer.after("top-bar"))
      .add_system_to_stage(UI_STAGE, round_summary::round_summary.after("top-bar"))
      .add_system_to_stage(UI_STAGE, ui_block_check);
  }
}

