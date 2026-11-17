//! The skate mode ([`Mode::Skate`](crate::Mode::Skate)): what a skating engine adds to the core.
//!
//! - [`SkatePark`]: the skating annotations over the core [`Stage`](crate::Stage): what's grindable, and which of a
//!   placed piece's faces are a rail's tube.
//! - [`SkaterLink`]: the rider and the board, each its own frame (they part: walking, bails, a board left behind);
//!   [`SkaterLink::focus`] is skating's mapping onto the core [`Focus`].
//! - [`TrickText`]: the trick being done and its points, for the runtime's plain readout when the engine draws no HUD.
//! - [`SkateMode`]: a skating engine's part of [`crate::EngineCore`].
//! - [`sound`]: what the skater did, for the skating sounds the runtime carries when the engine has none.
//! - [`body`]: a skater model's body groups ([`crate::model::BodyGroup`]): the rider and the board, and the board's
//!   contact shadow ([`body::BOARD_FOOTPRINT`]).
//!
//! Placing the skater is the core's [`PlacePlayer`](crate::PlacePlayer): a skating engine puts it on its board there.
//!
//! Everything is in the core's one space and unit ([`crate::stage`]). Skating words and ideas belong here, never in
//! the core; more arrives as the runtime's steps move skating out of the app (its `docs/game-code-out-plan.md`).
use crate::{Focus, Frame};
use glam::Vec3;
use std::ops::Range;

/// A skater model's body groups ([`crate::model::BodyGroup`]): the rider and the board move apart (walking, bails, a
/// board left behind), and each casts its own contact shadow.
pub mod body {
    use super::SKATER_HEIGHT;
    use crate::model::{Ellipse, GroupId};

    /// The rider: cast from its bones ([`crate::model::Cast::Segments`]).
    pub const RIDER: GroupId = 0;
    /// The board: cast as [`BOARD_FOOTPRINT`] along it, tail to nose.
    pub const BOARD: GroupId = 1;

    /// The board's contact shadow: the deck, then its two trucks (smaller and darker, where the wheels meet the
    /// ground), laid along it from its middle. Life-size metres (a deck about 0.8 m long) in skater heights.
    pub const BOARD_FOOTPRINT: [Ellipse; 3] = [
        Ellipse { along: 0.0, half_length: 0.42 / SKATER_HEIGHT, half_width: 0.14 / SKATER_HEIGHT, dark: 0.65 },
        Ellipse { along: 0.25 / SKATER_HEIGHT, half_length: 0.1 / SKATER_HEIGHT, half_width: 0.12 / SKATER_HEIGHT, dark: 0.55 },
        Ellipse { along: -0.25 / SKATER_HEIGHT, half_length: 0.1 / SKATER_HEIGHT, half_width: 0.12 / SKATER_HEIGHT, dark: 0.55 },
    ];
}

/// How tall a skater is at life size, metres.
pub const SKATER_HEIGHT: f32 = 1.8;

/// What's skated on the [`Stage`](crate::Stage) of the same `revision` beyond its solid faces. A skating engine is
/// given the park first (it keeps it) and then the stage, and takes the two together.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkatePark {
    /// The stage's it annotates.
    pub revision: u64,
    /// The room's grindable edges (furniture and table tops: ledges).
    pub room_edges: Vec<(Vec3, Vec3)>,
    /// The placed pieces' annotations, each of a solid of the stage (in the stage's order).
    pub pieces: Vec<ParkPiece>,
    /// Every grind rail as a polyline: the room's edges (two points each), then the pieces' rails, ledges and
    /// copings.
    pub rails: Vec<Vec<Vec3>>,
}

/// A placed piece's skating: which of its solid's quads are a rail's tube, and its grindable edges.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParkPiece {
    /// Its [`Solid`](crate::Solid)'s id.
    pub solid: u64,
    /// Its solid's quads that are a grind tube's faces (convex rail collision, metal); the rest are its body.
    pub rail_quads: Range<usize>,
    /// Its grindable edges (a box's or ledge's top edges).
    pub edges: Vec<(Vec3, Vec3)>,
}

impl SkatePark {
    /// The annotations of the stage's solid `id`.
    pub fn piece(&self, solid: u64) -> Option<&ParkPiece> {
        self.pieces.iter().find(|p| p.solid == solid)
    }
}

/// Where the skater is.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkaterLink {
    /// The rider: at its feet, facing where it faces.
    pub rider: Option<Frame>,
    /// The board: its deck's middle, its nose forward.
    pub board: Option<Frame>,
    /// The rider is on the board.
    pub on_board: bool,
    /// On a grind: the engine keeps its rails until it ends (it takes a new stage after).
    pub grinding: bool,
    /// How big the skater is drawn against life size (a game's world bigger than the room draws it smaller); 0:
    /// life size.
    pub size: f32,
}

impl SkaterLink {
    /// The skater's size against life size (1 unless the engine says otherwise).
    pub fn size(&self) -> f32 {
        if self.size > 0.0 { self.size } else { 1.0 }
    }

    /// The skater as the core's [`Focus`]: the rider, or the board when there's no rider; as tall as a skater at its
    /// size.
    pub fn focus(&self) -> Option<Focus> {
        let at = self.rider.or(self.board)?;
        Some(Focus { feet: at.position, up: at.up, forward: at.forward, height: SKATER_HEIGHT * self.size() })
    }
}

/// The trick text the runtime's plain readout shows (an engine without a HUD of its own gives it).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrickText {
    /// The trick in progress or just landed.
    pub name: String,
    /// The points it scored, if any.
    pub points: String,
    /// Score, combo, a new best.
    pub line: String,
}

/// A skating engine's part of [`crate::EngineCore`] ([`crate::EngineCore::skate`]).
pub trait SkateMode {
    /// The park over the stage of the same revision: kept until [`crate::EngineCore::set_stage`] brings that stage,
    /// and taken with it (the runtime offers the park first, every frame).
    fn set_park(&mut self, park: &SkatePark) -> crate::Applied;
    /// Where the rider and the board are after the last step: none while not in the runtime's world.
    fn link(&self) -> Option<SkaterLink>;
    /// The plain trick readout, for an engine with no HUD of its own.
    fn trick(&self) -> Option<TrickText> {
        None
    }
    /// What the skater did since the last ask, for the skating sounds the runtime carries: an engine with no sound of
    /// its own.
    fn sounds(&mut self) -> Option<sound::SkateSounds> {
        None
    }
}

/// What a skating engine's steps did, for the skating sounds the runtime carries (recordings of wheels, pops and
/// landings) when the engine has no sound of its own. The runtime picks the recordings, their levels and timing.
pub mod sound {
    use glam::Vec3;

    /// Something heard once.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Cue {
        /// Off the ground or a rail, by the skater's own doing.
        Pop,
        /// A flip trick started: the board's whoosh now, its catch a moment later if still in the air.
        Flip,
        /// Touchdown, with the speed into the surface (m/s; either sign).
        Land { impact: f32 },
        /// A push off the ground.
        Push,
        /// A step on foot, 0 to 1.
        Footstep { strength: f32 },
        Bail,
        /// Points banked, a multiplier up.
        Chime,
    }

    /// What's under the wheels.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub enum Rolling {
        /// Nothing: in the air, on foot, bailing.
        #[default]
        None,
        Ground,
        Grind,
    }

    /// The skater's sounds: the cues since the runtime last took them, in order, and how it is now.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct SkateSounds {
        /// Where the sounds come from (the core's space).
        pub at: Vec3,
        /// The board's speed, m/s.
        pub speed: f32,
        pub rolling: Rolling,
        /// Sliding sideways on its wheels (on the ground).
        pub powersliding: bool,
        /// In the air: a flip's catch is heard only while it is.
        pub airborne: bool,
        pub cues: Vec<Cue>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_focus_is_the_rider_or_else_the_board() {
        let frame = |x: f32| Frame { position: Vec3::new(x, 0.0, 0.0), forward: Vec3::Z, up: Vec3::Y };
        let mut link = SkaterLink { board: Some(frame(2.0)), ..Default::default() };
        assert_eq!(link.focus().map(|f| f.feet.x), Some(2.0));
        assert_eq!(link.focus().map(|f| f.height), Some(SKATER_HEIGHT));
        link.rider = Some(frame(1.0));
        link.size = 0.5;
        let focus = link.focus().unwrap();
        assert_eq!((focus.feet.x, focus.height, focus.forward), (1.0, SKATER_HEIGHT * 0.5, Vec3::Z));
        assert_eq!(SkaterLink::default().focus(), None);
    }

    #[test]
    fn a_park_piece_is_found_by_its_solid() {
        let park = SkatePark {
            pieces: vec![ParkPiece { solid: 3, rail_quads: 2..4, edges: vec![] }],
            ..Default::default()
        };
        assert_eq!(park.piece(3).map(|p| p.rail_quads.clone()), Some(2..4));
        assert!(park.piece(1).is_none());
    }
}
