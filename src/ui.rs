use bevy::prelude::*;
use bevy_ecs_tilemap::TilePos;
use bevy_egui::{egui, EguiContext, egui::Vec2 as EGVec2};

use crate::map::{DataLayer, OnMap, SelectedTile};
use crate::player::{PlayerStatus, PlayerIdentity};
use crate::utils::{pad_string_left, pad_string_right};
use crate::wizard_memory::{MemoryBlob, MemoryCell, MemoryLocation, MEMORY_SIZE};

#[derive(Debug, Component)]
pub struct BlockKeyInput;

#[derive(Debug)]
struct UIBlock {
  start: Vec2,
  size: Vec2,
}

impl UIBlock {
  fn is_inside(&self, point: Vec2) -> bool {
    point.x > self.start.x
      && point.x < self.start.x + self.size.x
      && point.y > self.start.y
      && point.y < self.start.y + self.size.y
  }
}

#[derive(Default, Debug)]
pub struct UIBlocks {
  top: Option<UIBlock>,
  right: Option<UIBlock>,
  left: Option<UIBlock>,
  memory: Option<UIBlock>,
  spells: Option<UIBlock>,
  popup: Option<UIBlock>,
}

impl UIBlocks {
  pub fn is_blocked(&self, point: Vec2) -> bool {
    self
      .top
      .as_ref()
      .map(|t| t.is_inside(point))
      .unwrap_or(false)
      || self
        .right
        .as_ref()
        .map(|t| t.is_inside(point))
        .unwrap_or(false)
      || self
        .left
        .as_ref()
        .map(|t| t.is_inside(point))
        .unwrap_or(false)
      || self
        .memory
        .as_ref()
        .map(|t| t.is_inside(point))
        .unwrap_or(false)
      || self
        .spells
        .as_ref()
        .map(|t| t.is_inside(point))
        .unwrap_or(false)
      || self
        .popup
        .as_ref()
        .map(|t| t.is_inside(point))
        .unwrap_or(false)
  }
}

#[derive(PartialEq)]
enum SidebarState {
  Big,
  Small,
}

impl Default for SidebarState {
  fn default() -> Self {
    SidebarState::Small
  }
}

#[derive(Component, Default)]
pub struct MemoryView {
  show: bool,
  current_value: (MemoryLocation, MemoryCell),
  rows_scrolled: usize,
}

pub fn memory_ui(
  mut gui: ResMut<EguiContext>,
  mut memory: ResMut<MemoryBlob>,
  mut blocks: ResMut<UIBlocks>,
  mut view: Query<&mut MemoryView>,
) {
  let mut view = view.get_single_mut().unwrap();
  if !view.show {
    return;
  }

  let window = gui.ctx_mut().available_rect();
  let frame_size = window.width() / 2.;
  blocks.memory = Some({
    UIBlock {
      start: Vec2::new(window.width() / 2., 0.),
      size: Vec2::new(window.width() / 2., window.height()),
    }
  });

  egui::SidePanel::right("memory viewer")
    .min_width(frame_size)
    .resizable(false)
    .show(gui.ctx_mut(), |ui| {
      ui.label("Memory");
      let total_rows = MEMORY_SIZE / 10;

      let mem_view = ui.group(|ui| {
        let mut start = view.rows_scrolled * 10;
        let total_height = ui.available_height() / 2.;
        let mut height = 0.;
        while height < total_height {
          let dh = ui
            .horizontal(|ui| {
              let mut row_label_text = format!("{}:", start);
              pad_string_right(&mut row_label_text, 8);
              ui.label(egui::RichText::new(row_label_text).monospace().weak());

              for index in start..start + 10 {
                if let Some(cell) = memory.get_one(MemoryLocation { pointer: index }) {
                  let mut cell_label_text = format!("{}", &cell.value);
                  pad_string_left(&mut cell_label_text, 3);
                  let mut styled_cell_label_text = egui::RichText::new(cell_label_text).monospace();
                  if index == view.current_value.0.pointer {
                    styled_cell_label_text = styled_cell_label_text.underline();
                  }
                  let k =
                    ui.add(egui::Label::new(styled_cell_label_text).sense(egui::Sense::click()));
                  if k.clicked() {
                    view.current_value = (MemoryLocation { pointer: index }, cell.to_owned());
                  }
                }
              }
            })
            .response
            .rect
            .height();

          height += dh;
          start += 10;
        }
      });
      // manually implementing scroll here because a custom controlled egui::ScrollArea seems really hard to use
      let y_scroll = mem_view.response.ctx.input().scroll_delta.y;
      if mem_view.response.hovered() && y_scroll != 0. {
        if y_scroll < 0. && view.rows_scrolled < total_rows - 1 {
          view.rows_scrolled += 1;
        } else if view.rows_scrolled > 1 {
          view.rows_scrolled -= 1;
        }
      }

      ui.label(format!("Address {}", view.current_value.0.pointer));
    });
}

pub fn left_panel(
  mut gui: ResMut<EguiContext>,
  mut blocks: ResMut<UIBlocks>,
  bevy_window: Res<Windows>,
  selected_tile: Query<(&TilePos), (With<SelectedTile>, With<DataLayer>, Without<OnMap>)>,
  player: Query<(&TilePos, &PlayerStatus, &PlayerIdentity)>
) {
  const PANEL_SIZE_FACTOR: f32 = 4.;
  let bevy_window = bevy_window.get_primary().unwrap();

  let eg_window = gui.ctx_mut().available_rect();
  blocks.left = Some(UIBlock {
    start: Vec2::new(0., 0.),
    size: Vec2::new(
      bevy_window.width() / PANEL_SIZE_FACTOR,
      bevy_window.height(),
    ),
  });
  let frame_size = eg_window.width() / PANEL_SIZE_FACTOR;

  egui::SidePanel::left("core_ui")
    .min_width(frame_size)
    .resizable(false)
    .show(gui.ctx_mut(), |ui| {
      let ui_height_tracker: f32 = 0.;

      ui.label("Your Turn");
      ui.label(egui::RichText::new("Actions").heading());

      ui.horizontal(|ui| {
        ui.vertical(|ui| {
          ui.label("Warrior");
        });
        ui.vertical(|ui| {
          ui.label("Wizard");
        });
      });

      ui.allocate_space(EGVec2::new(
        frame_size,
        (eg_window.height() / 3.) - ui_height_tracker,
      ));

      if let Ok(t) = selected_tile.get_single() {
        ui.label(format!("Tile ({}, {}) selected", t.0, t.1));
      } else {
        ui.label("No tile selected");
      }
    });
}
