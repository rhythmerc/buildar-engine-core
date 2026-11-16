# buildar-engine-core

The contract between an AR runtime and the game engines it hosts, as plain Rust data with no Bevy (crate `engine-core`).

The runtime is [skate3-quest-ar](https://github.com/rhythmerc/skate3-quest-ar), a Quest 3 app that brings game
engines into stages built in your real room. It owns the room, the stages, the controllers, the menus and all
drawing. An engine runs one game's simulation on the player's own game files and hands back what to draw and play:
[skate3-headless](https://github.com/rhythmerc/skate3-headless) (Skate 3) and
[thug2-headless](https://github.com/rhythmerc/thug2-headless) (THUG2).

## Core and modes

- **The core** (the crate root) is what every engine shares, whatever kind of game it is: controllers, the HUD as
  draw lists, and (as it grows) models and poses, audio, the stage, placing the player.
- **Each mode** is a module: `skate` now; racing, platforming and others later. Anything that only makes sense for
  one kind of game goes in its mode's module.
- **When in doubt, the mode module.** Moving something into the core later breaks nothing; moving it out breaks
  every engine. `tests/neutral_core.rs` fails if a mode's words show up in the core.

## What's here

- `Mode`, `TouchState`, `PadHeld`, `Control` (a controls-sheet row), `HudHand`, `Focus` (where the player is).
- `stage`: the one space and unit; `Stage` (the room and placed solids), `Applied`, `Frame`, `PlacePlayer`.
- `skate`: `SkatePark` (rails, edges, rail tubes over the stage), `SkaterLink` (rider and board), `TrickText`.
- `hud`: `HudFrame` (vertices, draws, parts), `Placement` (by a controller, at the player), shading and blending
  that cover both THUG2's fixed-function 2D and Skate 3's Flash-style colour transforms.

The rest (the engine trait, models and poses, audio) arrives with the
runtime's steps that first use it; see its `docs/game-code-out-plan.md`.

## Using it

```toml
engine-core = { git = "https://github.com/rhythmerc/buildar-engine-core", rev = "<sha>" }
```

`glam` is pinned to the version Bevy 0.18.1 uses (`=0.30.10`), so a Bevy host passes vectors across unconverted.

## Licence

GPL-3.0, the same as the runtime (see `LICENSE`).
