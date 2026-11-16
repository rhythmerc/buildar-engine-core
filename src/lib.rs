//! What an AR runtime and the game engines it hosts say to each other, as plain data (no Bevy).
//!
//! The runtime (the headset app) owns the room, the stages built in it, the controllers, the menus and all drawing.
//! An engine runs one game's simulation on the player's own game files and hands back what to draw and play.
//!
//! The crate root is the **core**: what every engine shares, whatever kind of game it is. Each **mode** (a kind of
//! game, with its own kind of stage) is a module: [`skate`] now; racing, platforming and others later. Anything that
//! only makes sense for one kind of game goes in that mode's module, never here: when in doubt, the mode module
//! (moving something into the core later breaks nothing; moving it out breaks every engine). The test
//! `tests/neutral_core.rs` keeps one mode's words out of the core.
//!
//! What's here so far (the rest arrives with the steps that first use it; see the runtime's
//! `docs/game-code-out-plan.md`):
//! - [`Mode`]: the kinds of game.
//! - [`TouchState`]: the controllers, as the runtime reads them.
//! - [`hud`]: an engine's HUD as draw lists, its parts and where the runtime puts them.
pub mod hud;
pub mod skate;

pub use glam;
use serde::{Deserialize, Serialize};

/// A kind of game, and so the kind of stage it's played on and built with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Skating ([`skate`]).
    Skate,
}

impl Mode {
    pub const ALL: [Mode; 1] = [Mode::Skate];

    pub fn name(self) -> &'static str {
        match self {
            Mode::Skate => "Skate",
        }
    }

    /// Its name in files (a stage's `mode` tag).
    pub fn key(self) -> &'static str {
        match self {
            Mode::Skate => "skate",
        }
    }

    pub fn from_key(key: &str) -> Option<Mode> {
        Mode::ALL.into_iter().find(|m| m.key() == key.trim())
    }
}

/// One frame of raw Touch controller state, OpenXR ranges: sticks -1..1 (+y forward), triggers and grips 0..1.
/// Each engine maps it to its own game's pad.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TouchState {
    pub left_stick: [f32; 2],
    pub right_stick: [f32; 2],
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub left_grip: f32,
    pub right_grip: f32,
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub left_stick_click: bool,
    pub right_stick_click: bool,
    pub menu: bool,
}

/// A controller, for what's placed by one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HudHand {
    Left,
    #[default]
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_keys_round_trip() {
        for mode in Mode::ALL {
            assert_eq!(Mode::from_key(mode.key()), Some(mode));
        }
        assert_eq!(Mode::from_key(" skate\n"), Some(Mode::Skate));
        assert_eq!(Mode::from_key("race"), None);
    }

    #[test]
    fn a_hand_is_saved_as_a_word() {
        // (players' settings files keep it: the spelling must not change)
        assert_eq!(serde_json::to_string(&HudHand::Left).unwrap(), "\"left\"");
        assert_eq!(serde_json::from_str::<HudHand>("\"right\"").unwrap(), HudHand::Right);
    }
}
