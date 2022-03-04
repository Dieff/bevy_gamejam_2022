# Space Wizard Power Tournament

Space Wizard Power Tournament is a very work-in-progress implementation of a turn-based
tactical role-playing game. It was built by a team of one for the first official [Bevy game jam](https://itch.io/jam/bevy-jam-1/entries).
Warning: it isn't very playable.

This project is my first ever game jam submission. It was also my first time using the Bevy game engine.
One thing I learned was how much I love Entity-Component-System architecture. The way that it allows you
to think directly about what the game state should be, instead of what methods to call on your classes,
is really helpful. I started this project having no idea how to organize or prepare for a game jam, and also
having no concept of how long it would take to get things working. That last I certainly learned the hard way.
Though the game is not even close to done, I wanted to submit what I had in order to give myself a baseline for
future game jams or Bevy projects. Next year, I'll make Fire Emblem look like Pacman!

The source code is hosted on [Github](https://github.com/Dieff/bevy_gamejam_2022) and 
[Source Hut](https://git.sr.ht/~dieff/bevy_gamejam_2022). There are a few comments here and there.

## Current Features

- Levels can be created in the [LDtk Level Designer Toolkit](https://ldtk.io/), and loaded into the game.
- You have a team of two units (one wizard, one warrior), with which to fight enemies. Choose their
  actions on the map. Unusually for the genre, units performs their actions at the exact same time,
  which could allow for cool combos or buff abilities.
- Enemies have a simple AI, and will attack the closest unit. There are several other enemy AI behaivours available in the code,
  including self-preservation, and the abilitiy to prioritize which player unit seems the weakest (and also a bug or two).

## Planned Features

The original intent of the game was to explore custom spell creation in an RPG. Those that I have played involve
unlocking premade spells as you level up. Instead, I though it would be cool to allow players to create their own
spells in the most flexible way possible: a programming langauge.

The idea was that spells would be short programs that would modify the game's state directly. Since Rust is a pretty high-level langauge,
I decided it would be unapproachable to mess with the game's actual memory (and very `unsafe`!), so I settled on a system
to serialize game objects into a virtual memory buffer. After a spell is cast, the game reads the buffer to see what has changed,
and then propagates the change on to the real state. The virtual memory also let's me do cool things like create memory cells
that hold a 10 digit number, instead of 8 bits, meaning that viewing and editing memory would be much easier for
non-programmers.

Players could unlock new type signatures, which would be highlited in a memory browser to show which part of the memory correspond to
which type. Then, they would developed new spells by trial and error in a training level. A few example spells could be included
to show the ropes. To keep things fair, changing different type fields would require different amounts of mana. It might take
less to change a tile's temperature than an enemy's position.

The main downfall of this idea what that I felt the game needed a basic level of complexity for it to be fun to write spells.
If the only state is enemy position and enemy health, you would win by writing `enemy.health = 0`. Not much fun... Implementing a turn-selection UI,
enemy behaivour, and types of map tiles took me way too long, so very little work on the spell system was actually done.

Also planned
- A special level selection screen: in keeping with the planned space theme, the levels would be represented as planets.
  A level preview screen would describe the conditions of the planet, and the enemy team you would be facing.
- Game event feed: in the bottom left of the UI, a feed would show the events in a turn. The would include enemy attacks,
  as well as comments by your characters (i.e. they say ouch when they get hit, or maybe offer suggestion to the player).
- Tutorial level: the first level should have a pop-up that explains how to play the game. Some further pop-ups could
  elaborate on the spell system, and help you create your first spell.
- Further cross-platform support. In theory, it should be possible to build the game for Windows, MacOS, Linux, and Web Assembly.


## Credits
This project is built using the [Bevy game engine](https://bevyengine.org/). A few other Rust crates were used,
see `Cargo.toml` for details.

The levels were created with the amazing [LDtk Level Designer Toolkit](https://ldtk.io/).

The tile sets contained in `assets/tiles/sample_tileset` are downloaded from [https://kenney.nl](https://kenney.nl),
and are licensed with CC0 1.0.
