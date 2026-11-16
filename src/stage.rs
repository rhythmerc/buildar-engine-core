//! What an engine's game is played on, and putting the player on it.
//!
//! **One space for everything here:** the runtime's world. Its frame is the room's as the runtime lays it out (y up,
//! right-handed), and its unit is the metre as the game sees it: the runtime may show the room bigger or smaller to
//! the game than it really is (the player's world scale), so a real metre can be more or less than one of these. The
//! [`Stage`], [`crate::Focus`], [`Frame`]s and [`PlacePlayer`] are all in it; an engine with units or an origin of its
//! own converts inside itself.
use glam::Vec3;

/// The solid surfaces a game is played on, in the runtime's world space ([module docs](self)), rebuilt whenever the
/// room or what's placed in it changes. Quads are kept as quads (corners in order; an engine may treat a flat quad
/// differently from two triangles).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stage {
    /// Changes on every rebuild (0: nothing built yet).
    pub revision: u64,
    /// The room's flat faces (floor, walls, furniture), as quads.
    pub room_quads: Vec<[Vec3; 4]>,
    /// The room's other faces (the scanned mesh where no flat face covers it), as triangles.
    pub room_triangles: Vec<[Vec3; 3]>,
    /// The solid objects placed in the room, each with an id kept while it stays placed.
    pub solids: Vec<Solid>,
}

/// One placed object's solid faces.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Solid {
    pub id: u64,
    pub quads: Vec<[Vec3; 4]>,
}

impl Stage {
    /// The room as triangles (no placed objects): each quad as (0, 1, 2) and (0, 2, 3), then the triangles.
    pub fn room(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        self.room_quads.iter().flat_map(|&q| split(q)).chain(self.room_triangles.iter().copied())
    }

    /// The room and the placed objects as triangles.
    pub fn triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        self.room().chain(self.solids.iter().flat_map(|s| s.quads.iter().flat_map(|&q| split(q))))
    }

    /// The placed object `id`.
    pub fn solid(&self, id: u64) -> Option<&Solid> {
        self.solids.iter().find(|s| s.id == id)
    }
}

/// A quad's two triangles: (0, 1, 2) and (0, 2, 3).
pub fn split([a, b, c, d]: [Vec3; 4]) -> [[Vec3; 3]; 2] {
    [[a, b, c], [a, c, d]]
}

/// Whether an engine took what it was given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    /// It has it (or had it already).
    Done,
    /// Not now: the runtime offers it again (the latest one) until it's taken. A game can't always change its ground
    /// mid-move.
    Deferred,
}

/// A place and the way it faces, in the runtime's world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub position: Vec3,
    /// Its forward and up: unit vectors, at right angles.
    pub forward: Vec3,
    pub up: Vec3,
}

/// Put the player down at `point` (a point on the stage's ground), facing along `forward` (its level part counts).
/// What "down" means is the engine's: on its feet, in its vehicle (a mode's module says what it means there).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacePlayer {
    pub point: Vec3,
    pub forward: Vec3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stage_is_its_room_and_solids_as_triangles() {
        let quad = [Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 1.0), Vec3::Z];
        let stage = Stage {
            room_quads: vec![quad],
            room_triangles: vec![[Vec3::ZERO, Vec3::Y, Vec3::X]],
            solids: vec![Solid { id: 4, quads: vec![quad, quad] }],
            ..Default::default()
        };
        assert_eq!(stage.room().count(), 3);
        assert_eq!(stage.triangles().count(), 7);
        assert_eq!(stage.room().next(), Some([quad[0], quad[1], quad[2]]));
        assert_eq!(stage.solid(4).map(|s| s.quads.len()), Some(2));
        assert!(stage.solid(5).is_none());
    }
}
