//! An engine's HUD, for the runtime to draw: the game's 2D as draw lists, cut into parts that the runtime places in
//! the room (by a controller, at the player).
//!
//! The engine decides what is drawn and how it's laid out; the runtime draws it (one renderer for every engine) and
//! puts each part where its [`Placement`] says. Positions are in the engine's own screen space, its **canvas**:
//! pixels, x right, y down, as the game drew them.
//!
//! A frame is filled in place ([`HudFrame::clear`] keeps its buffers), so an engine re-fills the same one each frame.
//! Textures are named by a [`TextureId`] an engine keeps for as long as the texture lives (never reusing one) and are
//! fetched once each.
use crate::HudHand;
use glam::Vec2;

/// A HUD texture's name, kept by the engine while the texture lives and never reused.
pub type TextureId = u32;

/// A texture's top level: `width` x `height` pixels of RGBA8, sRGB-encoded, straight (not premultiplied) alpha.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rgba8 {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// A vertex of a HUD draw.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HudVertex {
    /// Canvas pixels.
    pub position: Vec2,
    pub uv: Vec2,
    /// RGBA, as the game gives it (sRGB-encoded); see [`Shading`] for what it does.
    pub color: [u8; 4],
}

/// How a draw goes over what's under it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HudBlend {
    /// Over it by its alpha.
    #[default]
    Alpha,
    /// Added to it, by its alpha (what's under keeps its coverage).
    Add,
    /// Drawn as it is: its alpha taken as 1.
    Opaque,
}

/// How a draw's colour is made from its texture and vertices. Either way the result is clamped to 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shading {
    /// The vertices' colour times the texture's, times `scale` (none: the vertices' colour alone). With `gamma`,
    /// worked in sRGB-encoded values, as old fixed-function games did (THUG2's 2D: `scale` 2, `gamma` on).
    Modulate { scale: f32, gamma: bool },
    /// The texture's colour times `multiply` plus `add`, in linear values; the vertices' colour unused (a Flash-style
    /// colour transform, as Skate 3's HUD scripts give).
    Transform { multiply: [f32; 4], add: [f32; 4] },
}

impl Default for Shading {
    fn default() -> Self {
        Shading::Modulate { scale: 1.0, gamma: false }
    }
}

/// A triangle list of a frame's vertices with one texture: `count` vertices (three a triangle) from `first`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HudDraw {
    pub first: u32,
    pub count: u32,
    pub texture: Option<TextureId>,
    /// The [`HudPart`] it belongs to (its index in [`HudFrame::parts`]).
    pub part: u16,
    pub blend: HudBlend,
    pub shading: Shading,
}

/// Where a part goes in the room.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Placement {
    /// A panel by a controller. Real metres a canvas pixel; its `anchor` (a point of the panel: -0.5..0.5 across and
    /// up from its middle) put `offset` from the grip (real metres: right and up, as the player sees it).
    Hand {
        metres_per_px: f32,
        offset: Vec2,
        anchor: Vec2,
        /// Always by this controller (none: the one the player chose).
        hand: Option<HudHand>,
        /// Where it goes instead when the player chose the left controller (shared with what keeps to the left
        /// one): offset and anchor.
        on_left: Option<(Vec2, Vec2)>,
    },
    /// At the player in the room (its [`crate::Focus`]), turned to the eyes: above them or beside them. As wide as
    /// `width_per_height` of the player's drawn height, and never narrower than `least_view_share` of its distance
    /// from the eyes (so it reads from across the room).
    AtPlayer { side: Side, width_per_height: f32, least_view_share: f32 },
}

/// Which side of the player an [`Placement::AtPlayer`] part is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    /// Over the head, clear of it.
    #[default]
    Above,
    /// Beside the body, to the viewer's right.
    Beside,
}

/// A piece of the HUD with a picture and a place of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct HudPart {
    /// What it is ("score", "meter", ...): for logs and settings, not for meaning.
    pub name: String,
    /// The canvas area its picture shows, pixels.
    pub size: Vec2,
    /// The canvas point at the middle of its picture.
    pub view: Vec2,
    /// Nothing to show now (hidden).
    pub empty: bool,
    pub placement: Placement,
}

/// One frame of an engine's HUD.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudFrame {
    /// The canvas, pixels.
    pub canvas: Vec2,
    pub vertices: Vec<HudVertex>,
    /// In the order they're drawn.
    pub draws: Vec<HudDraw>,
    pub parts: Vec<HudPart>,
}

impl HudFrame {
    /// Emptied for the next frame, its buffers kept.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.draws.clear();
        self.parts.clear();
    }

    /// A draw's vertices.
    pub fn vertices_of(&self, draw: &HudDraw) -> &[HudVertex] {
        let first = draw.first as usize;
        self.vertices.get(first..first + draw.count as usize).unwrap_or(&[])
    }

    /// A part's draws, in order.
    pub fn draws_of(&self, part: u16) -> impl Iterator<Item = &HudDraw> {
        self.draws.iter().filter(move |d| d.part == part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_is_reused_and_read_by_part() {
        let mut frame = HudFrame { canvas: Vec2::new(640.0, 480.0), ..Default::default() };
        frame.vertices.extend([HudVertex::default(); 6]);
        frame.draws.push(HudDraw { first: 0, count: 3, part: 0, ..Default::default() });
        frame.draws.push(HudDraw { first: 3, count: 3, part: 1, blend: HudBlend::Add, ..Default::default() });
        assert_eq!(frame.vertices_of(&frame.draws[1]).len(), 3);
        assert_eq!(frame.draws_of(1).count(), 1);
        let capacity = frame.vertices.capacity();
        frame.clear();
        assert!(frame.vertices.is_empty() && frame.draws.is_empty());
        assert_eq!(frame.vertices.capacity(), capacity);
    }

    #[test]
    fn a_draw_past_the_vertices_has_none() {
        let frame = HudFrame::default();
        assert!(frame.vertices_of(&HudDraw { first: 5, count: 3, ..Default::default() }).is_empty());
    }
}
