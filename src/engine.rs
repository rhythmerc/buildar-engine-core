//! An engine as the runtime drives it ([`EngineCore`]): one game's simulation, made on a worker by its
//! [`EngineFactory`], then given the controllers, the stage and where to put the player, stepped by its own period,
//! and asked after each step what to draw and play.
//!
//! The runtime owns the clock: it steps an engine a whole number of [`EngineCore::period`]s behind real time (a few at
//! most a frame), not while the player is busy elsewhere ([`PadHeld::pause`]), and numbers and stamps each step's poses
//! itself ([`Pose::generation`], [`Pose::step`]). An engine says only what a step did.
use crate::audio::{Clip, StreamFactory, Voice};
use crate::hud::{HudFrame, Rgba8, TextureId};
use crate::model::{Model, Pose};
use crate::stage::{Applied, PlacePlayer, Stage};
use crate::{Focus, PadHeld, TouchState};
use std::time::Duration;

/// Makes an engine on a worker thread (slow work, such as loading the game's files, happens here).
/// What the runtime knows of an engine before one is made: each engine crate exports one (as `INFO`), and the
/// runtime registers it with the engine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EngineInfo {
    /// How many of its game's metres a real metre is at the player's x1.0 size: the size its player looks right at in a
    /// room (a skater a few hand-spans tall, a kart toy-sized). The player sizes a course up or down from it.
    pub natural_scale: f32,
}

pub type EngineFactory = Box<dyn FnOnce() -> Result<Box<dyn EngineCore>, String> + Send>;

/// One game's simulation, hosted by the runtime. Each mode adds its own part ([`EngineCore::skate`]).
pub trait EngineCore: Send {
    /// How long its next step is. Read again after every step (a game may change its own clock).
    fn period(&self) -> Duration;

    /// The controllers, for the steps until the next call: the engine maps them to its game's pad.
    fn set_input(&mut self, touch: &TouchState, held: PadHeld);

    /// What the player sees from (their eyes in the stage's space: right, up and forward), for a game whose stick is
    /// relative to its camera.
    fn set_view(&mut self, _eyes: Option<[glam::Vec3; 3]>) {}

    /// How many of the stage's metres a real metre is (the player's world scale: the room shown bigger to the game,
    /// or smaller), offered every frame. An engine that judges real-world sizes, such as a scan's noise or a
    /// tolerance in real centimetres, uses it to turn them into the stage's; most need nothing from it.
    fn set_world_scale(&mut self, _stage_metres_per_real: f32) {}

    /// The latest stage, offered every frame: taken now, or later when the game can ([`Applied::Deferred`]).
    fn set_stage(&mut self, stage: &Stage) -> Applied;

    /// Put the player down there (now, or with the stage it takes next).
    fn place_player(&mut self, place: PlacePlayer);

    /// The most triangles (counted as [`Stage::triangles`]) its game's collision can take, if it has a limit: the
    /// runtime offers it the stage [within](Stage::within) that many, the scanned mesh farthest from its player left
    /// out.
    fn stage_triangles_most(&self) -> Option<usize> {
        None
    }

    /// Whether its player can be put in the runtime's world now. A game still loading after it is made (a race that
    /// loads on its own thread) says no until it can take one; the runtime asks each frame, and tells the player to
    /// place theirs once it says yes.
    fn ready_to_place(&self) -> bool {
        true
    }

    /// Whether it steps now: a game may wait (for a stage of the runtime's to be in).
    fn wants_steps(&self) -> bool {
        true
    }

    /// One step.
    fn step(&mut self) -> Result<(), String>;

    /// Where the player is after the last step: none while not in the runtime's world.
    fn focus(&self) -> Option<Focus>;

    /// Its models, once (after it is made).
    fn models(&mut self) -> Vec<Model>;

    /// The poses its last step left, each saying its model and whether it cut (the runtime numbers and stamps them).
    fn poses(&mut self, out: &mut Vec<Pose>);

    /// The voices playing after its last step ([`crate::audio`]).
    fn voices(&mut self, _out: &mut Vec<Voice>) {}

    /// A clip's samples, asked once each.
    fn clip(&mut self, _id: u32) -> Option<Clip> {
        None
    }

    /// Its own mix, asked once: the runtime makes and renders it on its thread.
    fn stream(&mut self) -> Option<StreamFactory> {
        None
    }

    /// Whether its HUD is seen, before the frame's steps (a game may only draw it then).
    fn want_hud(&mut self, _wanted: bool) {}

    /// Its HUD after the frame's steps, into `out`: true when it is to be drawn again (false: unchanged, or none).
    fn hud(&mut self, _out: &mut HudFrame) -> bool {
        false
    }

    /// A HUD texture's pixels, asked once each.
    fn hud_texture(&mut self, _id: TextureId) -> Option<Rgba8> {
        None
    }

    /// One of its settings, by its key (what they are is the engine's; the runtime keeps the player's choices and gives
    /// them again each time the engine is made). A key it doesn't know is ignored.
    fn set_setting(&mut self, _key: &str, _value: f32) {}

    /// The skate mode's part, for an engine of that mode.
    fn skate(&mut self) -> Option<&mut dyn crate::skate::SkateMode> {
        None
    }
}
