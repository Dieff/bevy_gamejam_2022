use bevy::prelude::*;
use bevy_ecs_tilemap::TilePos;
use bevy_egui::{egui, egui::Vec2 as EGVec2, EguiContext};

use crate::constants;
use crate::map::{DataLayer, DrawOnMap, SelectedTile};
use crate::player::{MapEntityType, PlayerStatus, PlayerType};
use crate::spells::AvailableSpell;
use crate::turn::{
  EntityAction, EntityPendingAction, PlayerActionChosen, StartTurn, TurnStatus, TurnUIState,
};

type PlayerStatusQuery<'a> = (
  Entity,
  &'a TilePos,
  &'a PlayerStatus,
  &'a MapEntityType,
  Option<&'a EntityPendingAction>,
);

type SpellList<'a> = Vec<&'a str>;

fn draw_single_bar(
  ui: &mut egui::Ui,
  max: f32,
  cur: f32,
  high_color: egui::Color32,
  low_color: egui::Color32,
) -> egui::Rect {
  let width = ui.available_width();

  egui::plot::Plot::new("Health")
    .allow_drag(false)
    .allow_zoom(false)
    .width(width - 5.)
    .height(30.)
    .show_axes([false, true])
    .include_x(max)
    .show(ui, |pui| {
      let ratio = cur / max;
      assert!(ratio <= 1.);
      let color = if ratio < 0.3 { low_color } else { high_color };

      let chart = egui::plot::BarChart::new(vec![egui::plot::Bar::new(0., cur.into()).fill(color)])
        .horizontal()
        .width(20.);
      pui.bar_chart(chart);
    })
    .response
    .rect
}

#[derive(Debug, PartialEq, Clone)]
enum UIPlayerAction {
  Wait,
  Move,
  Attack,
  Cast,
}

// If an entity is returned, it means that the entity (player) has cancelled the action
fn draw_player_status<'a>(
  player_status: PlayerStatusQuery<'a>,
  player_type: PlayerType,
  spells: &SpellList,
  ui: &mut egui::Ui,
) -> Option<(Entity, EntityPendingAction)> {
  let (p_entity, _, status, _, pending_action) = player_status;
  let mut return_val = None;

  let height = ui.available_height();

  ui.add_space(10.);
  ui.label("Health:");
  draw_single_bar(
    ui,
    constants::PLAYER_MAX_HEALTH,
    status.health,
    egui::Color32::GREEN,
    egui::Color32::RED,
  )
  .height();
  ui.add_space(10.);

  if let Some(magika) = status.magika {
    ui.label("Magika: ");
    draw_single_bar(
      ui,
      constants::PLAYER_MAX_MAGIKA,
      magika,
      egui::Color32::BLUE,
      egui::Color32::BLUE,
    )
    .height();
  }
  let used_height = height - ui.available_height();
  ui.add_space((height / 1.8) - used_height);

  ui.label(egui::RichText::new("Actions").strong());
  ui.vertical(|ui| {
    let ui_action = match pending_action.map(|j| &j.action) {
      Some(EntityAction::Move(_)) => UIPlayerAction::Move,
      Some(EntityAction::Wait) => UIPlayerAction::Wait,
      Some(EntityAction::Cast(_)) => UIPlayerAction::Cast,
      Some(EntityAction::Attack) => UIPlayerAction::Attack,
      _ => UIPlayerAction::Wait,
    };

    let mut selection = ui_action.clone();

    egui::ComboBox::from_id_source(player_type)
      .selected_text(format!("{:?}", ui_action))
      .show_ui(ui, |ui| {
        ui.selectable_value(&mut selection, UIPlayerAction::Wait, "Wait");
        ui.selectable_value(&mut selection, UIPlayerAction::Move, "Move");
        ui.selectable_value(&mut selection, UIPlayerAction::Attack, "Attack");
        if player_type == PlayerType::Wizard {
          ui.selectable_value(&mut selection, UIPlayerAction::Cast, "Cast");
        }
      })
      .inner;

    let mut new_spell = "".to_owned();
    if let Some(cur_spell) = pending_action.unwrap().get_spell() {
      let start_index = spells
        .iter()
        .enumerate()
        .find(|(_, spell_name)| **spell_name == cur_spell)
        .map(|(i, _)| i)
        .unwrap_or(0);
      let mut selected_spell = start_index;
      egui::ComboBox::from_id_source("Spells").show_index(
        ui,
        &mut selected_spell,
        spells.len(),
        |i| spells[i].to_owned(),
      );
      if selected_spell != start_index {
        new_spell = spells[selected_spell].to_string();
        return_val = Some((
          p_entity,
          EntityPendingAction {
            action: EntityAction::Cast(new_spell.clone()),
            is_ready: true,
          },
        ));
      }
    }

    if selection != ui_action {
      dbg!(&selection, ui_action);
      return_val = Some((
        p_entity,
        match selection {
          UIPlayerAction::Attack => EntityPendingAction {
            action: EntityAction::Attack,
            is_ready: false,
          },
          UIPlayerAction::Cast => EntityPendingAction {
            action: EntityAction::Cast(new_spell),
            is_ready: false,
          },
          UIPlayerAction::Move => EntityPendingAction {
            action: EntityAction::Move((TilePos::default(), TilePos::default())),
            is_ready: false,
          },
          UIPlayerAction::Wait => EntityPendingAction {
            action: EntityAction::Attack,
            is_ready: true,
          },
        },
      ));
    }
  });

  return_val
}

// Does not mutate components directly, instead sends events
pub fn left_panel(
  mut gui: ResMut<EguiContext>,
  //mut blocks: ResMut<UIBlocks>,
  ui_state: Res<TurnUIState>,
  mut turn_event_writer: EventWriter<StartTurn>,
  mut cancel_event_writer: EventWriter<PlayerActionChosen>,
  selected_tile: Query<&TilePos, (With<SelectedTile>, With<DataLayer>, Without<DrawOnMap>)>,
  player_q: Query<PlayerStatusQuery>,
  spells: Query<&AvailableSpell>,
) {
  const PANEL_SIZE_FACTOR: f32 = 4.;

  let eg_window = gui.ctx_mut().available_rect();
  let frame_width = eg_window.width() / PANEL_SIZE_FACTOR;
  let frame_height = eg_window.height();

  let wizard = player_q
    .iter()
    .find(|(_, _, _, kind, _)| **kind == MapEntityType::Player(PlayerType::Wizard));
  let warrior = player_q
    .iter()
    .find(|(_, _, _, kind, _)| **kind == MapEntityType::Player(PlayerType::Warrior));

  let available_spells: SpellList = spells.iter().map(|s| s.name.as_str()).collect();

  egui::SidePanel::left("core_ui")
    .min_width(frame_width)
    .resizable(false)
    .show(gui.ctx_mut(), |ui| {
      if let (Some(wizard), Some(warrior)) = (wizard, warrior) {
        let actions_enabled = ui_state.status == TurnStatus::PlayerChoosing;

        let top_text = match ui_state.status {
          TurnStatus::PlayerChoosing => "Choose Your Actions",
          TurnStatus::PlayerRunning => "Your Turn",
          TurnStatus::EnemyRunning => "Enemy Turn",
          _ => "",
        };
        ui.label(egui::RichText::new(top_text).heading());
        ui.horizontal(|ui| {
          ui.set_max_height(frame_height / 3.);
          ui.set_max_width(frame_width / 2.);

          ui.vertical(|ui| {
            ui.allocate_space(EGVec2::new(frame_width / 2., 1.));
            let mut wl = egui::RichText::new("Warrior");
            if selected_tile
              .get_single()
              .ok()
              .filter(|pos| *pos == warrior.1)
              .is_some()
            {
              wl = wl.underline();
            }
            ui.label(wl);
            if let Some((player_entity, action_type)) =
              draw_player_status(warrior, PlayerType::Warrior, &available_spells, ui)
            {
              dbg!(&action_type);
              cancel_event_writer.send(PlayerActionChosen {
                player: player_entity,
                action_type,
              });
            }
          });
          ui.separator();
          ui.vertical(|ui| {
            ui.allocate_space(EGVec2::new(frame_width / 2., 1.));
            let mut wl = egui::RichText::new("Wizard");
            if selected_tile
              .get_single()
              .ok()
              .filter(|pos| *pos == wizard.1)
              .is_some()
            {
              wl = wl.underline();
            }
            ui.label(wl);
            if let Some((player_entity, action_type)) =
              draw_player_status(wizard, PlayerType::Wizard, &available_spells, ui)
            {
              cancel_event_writer.send(PlayerActionChosen {
                player: player_entity,
                action_type,
              });
            }
          });
        });

        ui.allocate_space(EGVec2::new(
          frame_width,
          (frame_height / 3.) - (frame_height - ui.available_height()),
        ));

        ui.vertical_centered(|ui| {
          let go = ui.add_enabled(actions_enabled, egui::Button::new("End Turn"));
          if go.clicked() {
            turn_event_writer.send(StartTurn(TurnStatus::PlayerRunning));
            dbg!("execute player turn");
          }
        });

        ui.separator();

        if let Ok(t) = selected_tile.get_single() {
          ui.label(format!("Tile ({}, {}) selected", t.0, t.1));
        } else {
          ui.label("No tile selected");
        }
      } else {
        ui.label("Loading..");
      }
    });
}
