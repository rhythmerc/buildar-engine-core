//! An engine's sound, and how the runtime plays it in the room.
//!
//! Two ways; an engine uses either or both:
//! - **Voices:** the engine plays clips, and after each step says which voices are playing ([`Voice`]): from where, how
//!   loud and how fast. The runtime mixes them in the room, each from its place. A clip's samples ([`Clip`]) are
//!   fetched once by its id.
//! - **A stream:** the engine mixes its own sound, a block at a time ([`Stream`]), on a thread the runtime runs and
//!   paces. The stream is made on that thread ([`StreamFactory`]), so it needn't be `Send`; what the engine's steps tell
//!   it travels however the engine likes (a channel the factory took with it). The runtime plays it from where the
//!   engine says.
//!
//! A mode may add its own kind (the skate mode's [`crate::skate::sound`]: what happened, for sounds the runtime
//! carries).
use glam::Vec3;
use std::time::Duration;

/// A clip's samples: 16-bit PCM, interleaved by channel.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Clip {
    pub channels: u16,
    /// Frames a second.
    pub rate: u32,
    pub samples: Vec<i16>,
}

/// One clip playing, as the engine left it after a step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Voice {
    /// The same while the engine keeps this voice (it may play other clips over time).
    pub id: u32,
    /// The clip it plays ([`Clip`], fetched by this id once).
    pub clip: u32,
    /// Changes each time the voice is played again from its start: a new sound, not the old one going on.
    pub starts: u32,
    /// Loops until the engine stops it; otherwise it plays once and ends by itself.
    pub looping: bool,
    /// Its level, 0 (silent) to 1 (full), with whatever fall-off the engine wants heard.
    pub gain: f32,
    /// How fast it plays: 1 is as recorded.
    pub speed: f32,
    /// Where it's heard from (the stage's space: [`crate::stage`]).
    pub at: Vec3,
}

/// An engine's own mix, rendered a block at a time on the runtime's thread for it.
pub trait Stream {
    /// Channels a frame (interleaved).
    fn channels(&self) -> u16;
    /// Frames a second.
    fn rate(&self) -> u32;
    /// How long one block plays: the runtime asks for one a block's time.
    fn block(&self) -> Duration;
    /// The next block. `paused`: the runtime holds the engine (a menu) and its steps have stopped; what a held game
    /// sounds like is the stream's to say.
    fn render(&mut self, paused: bool) -> Result<Vec<f32>, String>;
}

/// Makes an engine's [`Stream`], on the thread that will render it (slow work, such as decoding, can happen here).
pub type StreamFactory = Box<dyn FnOnce() -> Result<Box<dyn Stream>, String> + Send>;
