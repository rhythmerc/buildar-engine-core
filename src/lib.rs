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
//! - [`TouchState`]: the controllers, as the runtime reads them; [`PadHeld`]: whether the runtime has them. Each
//!   engine maps them to its own game's pad, and lists what does what as [`Control`]s.
//! - [`stage`]: what the game is played on ([`Stage`]), whether the engine took it ([`Applied`]), and putting the player
//!   on it ([`PlacePlayer`]); it also says the one space and unit everything here is in.
//! - [`Focus`]: where the player is, for the runtime to follow.
//! - [`hud`]: an engine's HUD as draw lists, its parts and where the runtime puts them.
//! - [`model`]: an engine's models ([`Model`]: a glTF file, or meshes and plain materials), their poses ([`Pose`]), when
//!   each was made ([`Step`]: the runtime draws between a model's last two) and the bodies they're made of, each
//!   casting its own contact shadow.
//! - [`audio`]: an engine's sound: the clips it plays as voices, each from its place ([`audio::Voice`]), or its own mix
//!   as a stream rendered on the runtime's thread ([`audio::Stream`]).
pub mod audio;
pub mod hud;
pub mod model;
pub mod skate;
pub mod stage;

pub use model::{Model, ModelId, ModelSource, Pose, Step};

pub use stage::{Applied, Frame, PlacePlayer, Solid, Stage};

pub use glam;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

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

/// Whether the runtime has the controllers this frame, beside the [`TouchState`] read from them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PadHeld {
    /// The runtime's (a menu open, building, aiming where to put the player): the engine's pad is neutral.
    pub held: bool,
    /// The player is busy elsewhere (a menu open, building): the engine may pause its game.
    pub pause: bool,
}

/// One row of an engine's controls sheet: what to press, and what it does in the game. The runtime lists its own
/// rows (its menu, placing the player) after the engine's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Control {
    /// The Touch controls, as the player reads them ("Right stick", "A / B").
    pub input: Cow<'static, str>,
    pub action: Cow<'static, str>,
}

impl Control {
    pub const fn new(input: &'static str, action: &'static str) -> Self {
        Self { input: Cow::Borrowed(input), action: Cow::Borrowed(action) }
    }
}

/// Where the player is in the room, which way it faces and how big it's drawn: what the runtime follows (what's placed
/// at the player, its sounds, the menus' distance checks, carrying the player over to another engine). In the
/// runtime's world space ([`stage`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Focus {
    /// The point it stands on.
    pub feet: Vec3,
    /// Its up (a unit vector): along it from `feet` to the top of it.
    pub up: Vec3,
    /// Where it faces (a unit vector).
    pub forward: Vec3,
    /// How tall it is drawn, world units.
    pub height: f32,
}

impl Focus {
    /// Its middle: halfway up it.
    pub fn middle(&self) -> Vec3 {
        self.feet + self.up * (self.height * 0.5)
    }
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
    fn a_focus_middle_is_halfway_up() {
        let focus = Focus { feet: Vec3::new(1.0, 0.0, 2.0), up: Vec3::Y, forward: Vec3::Z, height: 1.8 };
        assert_eq!(focus.middle(), Vec3::new(1.0, 0.9, 2.0));
    }

    #[test]
    fn a_hand_is_saved_as_a_word() {
        // (players' settings files keep it: the spelling must not change)
        assert_eq!(serde_json::to_string(&HudHand::Left).unwrap(), "\"left\"");
        assert_eq!(serde_json::from_str::<HudHand>("\"right\"").unwrap(), HudHand::Right);
    }
}
