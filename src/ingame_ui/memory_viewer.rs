use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::spells::wizard_memory::{MemoryLocation, MemoryCell, MemoryBlob, MEMORY_SIZE};
use crate::utils::{pad_string_left, pad_string_right};

//use super::basic_types::{UIBlocks, UIBlock};

#[derive(Component, Default)]
pub struct MemoryWindowState {
  current_value: (MemoryLocation, MemoryCell),
  rows_scrolled: usize,
}

pub fn memory_ui(
  mut gui: ResMut<EguiContext>,
  mut memory: ResMut<MemoryBlob>,
  //mut blocks: ResMut<UIBlocks>,
  mut view: Query<&mut MemoryWindowState>,
) {
  if view.get_single().is_err() {
    return;
  }
  let mut view = view.single_mut();

  let window = gui.ctx_mut().available_rect();
  let frame_size = window.width() / 2.;
  /*blocks.memory = Some({
    UIBlock {
      start: Vec2::new(window.width() / 2., 0.),
      size: Vec2::new(window.width() / 2., window.height()),
    }
  });*/

  egui::Window::new("Memory Viewer")
    .min_width(frame_size)
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


