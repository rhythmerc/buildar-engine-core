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

    /// The latest stage, offered every frame: taken now, or later when the game can ([`Applied::Deferred`]).
    fn set_stage(&mut self, stage: &Stage) -> Applied;

    /// Put the player down there (now, or with the stage it takes next).
    fn place_player(&mut self, place: PlacePlayer);

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

    /// Its HUD after the frame's steps; false: it has none.
    fn hud(&mut self, _out: &mut HudFrame) -> bool {
        false
    }

    /// A HUD texture's pixels, asked once each.
    fn hud_texture(&mut self, _id: TextureId) -> Option<Rgba8> {
        None
    }

    /// The skate mode's part, for an engine of that mode.
    fn skate(&mut self) -> Option<&mut dyn crate::skate::SkateMode> {
        None
    }
}
