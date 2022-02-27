use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use crate::utils::{pad_string_left, pad_string_right};
use crate::wizard_memory::{MemoryBlob, MemoryCell, MemoryLocation, MEMORY_SIZE};

/*struct UIBlocks 


struct UIBlock {
  start: Vec2,
  size: Vec2,
}

pub fn is_cursor_blocked<'a, I: Iterator<Item=&'a UIBlock>>(blocks: I, cursor: Vec2) -> bool {
  for block in blocks {
    if cursor.x > block.start.x && cursor.x < block.start.x + block.size.x {
      if cursor.y > block.start.y && cursor.y < block.start.y + block.size.y {
        return true;
      }
    }
  }
  false
} */

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
  sidebar: SidebarState,
  current_value: (MemoryLocation, MemoryCell),
  rows_scrolled: usize,
}


pub fn memory_ui(
  mut gui: ResMut<EguiContext>,
  memory: ResMut<MemoryBlob>,
  mut view: Query<&mut MemoryView>,
) {
  let mut view = view.get_single_mut().unwrap();
  let (frame_size, expand_button_text) = if view.sidebar == SidebarState::Big {
    (gui.ctx_mut().available_rect().width() / 2., "minimize")
  } else {
    (gui.ctx_mut().available_rect().width() / 5., "maximize")
  };

  egui::SidePanel::right("memory viewer")
    .min_width(frame_size)
    .resizable(false)
    .show(gui.ctx_mut(), |ui| {
      let expand_button = ui.add(egui::Button::new(expand_button_text));
      if expand_button.clicked() {
        if view.sidebar == SidebarState::Small {
          view.sidebar = SidebarState::Big;
        } else {
          view.sidebar = SidebarState::Small;
        }
      }

      if view.sidebar == SidebarState::Big {
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
                    let mut styled_cell_label_text =
                      egui::RichText::new(cell_label_text).monospace();
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
      }
    });
}
