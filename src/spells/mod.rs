use bevy::prelude::*;

pub mod wizard_lang;
pub mod wizard_memory;
pub mod wizard_types;

#[derive(Component, Debug, Clone)]
pub struct AvailableSpell {
  pub name: String,
  pub desc: String,
}

fn spawn_spells(mut commands: Commands) {
  let spells = vec![
    ("Test1", "The first test spell"),
    ("Test2", "The second test spell"),
  ];

  spells
    .into_iter()
    .map(|(name, desc)| AvailableSpell {
      name: name.to_owned(),
      desc: desc.to_owned(),
    })
    .for_each(|spell| {
      commands.spawn().insert(spell);
    });
}

pub struct SpellsPlugin;

impl Plugin for SpellsPlugin {
  fn build(&self, app: &mut App) {
    app.add_startup_system(spawn_spells);
  }
}
