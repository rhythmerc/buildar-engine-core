//! An engine's models and their poses, for the runtime to draw (one renderer for every engine) and to cast contact
//! shadows from.
//!
//! An engine describes each model once ([`Model`]): where its look comes from ([`ModelSource`]: a glTF file, or meshes
//! and textures as data with plain [`MaterialDesc`]s the runtime's own lighting draws), the shape of its skeleton, and
//! the bodies it is made of that can move apart ([`BodyGroup`]s). Then, after each of its steps, it gives the model's
//! [`Pose`]: which model it is ([`ModelId`]), where the model is and each joint's matrix, and when its step was
//! ([`Step`]). The runtime binds the source to the joints, carries the pose between steps (the headset draws faster
//! than an engine steps) and casts each body's shadow.
//!
//! **Between steps.** For each model the runtime keeps the last two poses it was given (each taken once, by
//! `generation`) and draws between them, [`Step::alpha`] of the way from the older to the newer: the time since the
//! newer one's step, in its periods, on the clock the runtime steps the engine by. So an engine that steps at a fixed
//! rate moves smoothly however fast the headset draws, with nothing to do but number its poses, say when its step was
//! and how long it is, and flag a cut. Nothing is drawn between two poses across a `discontinuity`, a change of the
//! period, or of the joint count: the newer one is drawn as it is.
//!
//! Everything is in the core's one space and unit ([`crate::stage`]).
use crate::hud::Rgba8;
use glam::{Mat4, Vec3};
use std::time::Duration;

/// A model an engine draws, described once.
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub source: ModelSource,
    /// Each joint's parent joint (`None`: a root of the skeleton), indexed as the pose's joints; empty when the
    /// engine doesn't know its skeleton's shape.
    pub parents: Vec<Option<u16>>,
    /// The bodies it is made of that can move apart, each casting its own contact shadow; empty: one body (the
    /// runtime casts it from the player's [`crate::Focus`]).
    pub groups: Vec<BodyGroup>,
}

/// Where a model's look comes from.
#[derive(Clone, Debug, PartialEq)]
pub enum ModelSource {
    /// A glTF file (`path`: as the runtime's asset reader finds it), drawn with its own materials. Its skins' joints
    /// are the pose's by node name (any case): the node named `joint_names[i]` is joint `i`, and a node with no name
    /// there is not posed. A joint's matrix in the [`Pose`] times `basis` is its node's, in the model's space; a node's
    /// local transform is that, relative to the nearest ancestor that is a joint.
    Gltf { path: String, joint_names: Vec<String>, basis: Mat4 },
    /// Meshes and their textures as data. Skinned with identity inverse binds: a posed vertex is the weighted sum of
    /// its joints' [`Pose`] matrices, each times the vertex as given. `joint_count` joints are posed (a joint past
    /// the pose's is the identity).
    Meshes { meshes: Vec<MeshData>, textures: Vec<Rgba8>, joint_count: u16 },
}

/// A mesh given as data: one entry a vertex in each attribute, and a triangle list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// Linear RGBA multipliers of its colour; empty: none.
    pub colors: Vec<[f32; 4]>,
    /// Up to four joints a vertex, and their weights (summing to 1).
    pub joints: Vec<[u16; 4]>,
    pub weights: Vec<[f32; 4]>,
    /// Three a triangle; its front is the side its corners go round counter-clockwise.
    pub indices: Vec<u16>,
    pub material: MaterialDesc,
}

/// How a mesh given as data is coloured and lit: plain data the runtime's lighting models draw (no shaders).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialDesc {
    /// RGBA, sRGB-encoded, straight alpha: multiplied with the texture and the vertex colours.
    pub base_color: [f32; 4],
    /// Its model's texture (an index into [`ModelSource::Meshes`]'s `textures`).
    pub texture: Option<u32>,
    pub alpha: AlphaMode,
    /// Both sides drawn (otherwise only the front).
    pub two_sided: bool,
    /// Its mesh's vertex colours count.
    pub vertex_color: bool,
    pub lighting: Lighting,
}

impl Default for MaterialDesc {
    fn default() -> Self {
        Self {
            base_color: [1.0; 4],
            texture: None,
            alpha: AlphaMode::Opaque,
            two_sided: false,
            vertex_color: false,
            lighting: Lighting::Standard,
        }
    }
}

/// How a material's alpha is used.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AlphaMode {
    /// Ignored.
    #[default]
    Opaque,
    /// Drawn where it is at least this (0..1), not elsewhere.
    Mask(f32),
    /// Blended over what's behind it.
    Blend,
}

/// Which of the runtime's lighting models draws a material.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lighting {
    /// Lit by the room's light, as everything else the runtime draws.
    #[default]
    Standard,
    /// Its colour as it is.
    Unlit,
}

/// Which of an engine's models a pose is of: its own numbering (an engine with one model: 0). The runtime keeps each
/// model's poses apart, and forgets a model that a step leaves out.
pub type ModelId = u32;

/// A model's pose after an engine's step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pose {
    /// The model it is of.
    pub model: ModelId,
    /// Changes with each new pose of the model (the runtime takes a pose once; a pose given again with the same
    /// generation is ignored, whatever else it says).
    pub generation: u64,
    /// Not to be blended from the one before (a cut: put down somewhere else, a teleport, a change of size).
    pub discontinuity: bool,
    /// The step that made it: when, and how long until the next ([`Step`]).
    pub step: Step,
    /// The model's space in the world.
    pub root: Mat4,
    /// Each joint's matrix in the model's space (what that means for each source: [`ModelSource`]).
    pub joints: Vec<Mat4>,
}

impl Pose {
    /// A point carried by joint `point.joint`, in the world ([`Point`]).
    pub fn point(&self, point: Point) -> Option<Vec3> {
        let joint = self.joints.get(usize::from(point.joint))?;
        Some(self.root.transform_point3(joint.transform_point3(point.at)))
    }
}

/// When an engine's step was, on the clock the runtime steps it by, and how long its steps are.
///
/// The runtime runs an engine's steps from a clock of its own (each step a `period` of it; the engine's own time, so a
/// slowed or paused game's clock runs slow or stands still). A pose's step says where on that clock it was made; the
/// runtime draws it between the pose before and itself by how far the clock has gone on since ([`Step::alpha`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Step {
    /// When the step ended: the clock's time when the pose was made (the steps' periods so far, summed).
    pub at: Duration,
    /// How long the engine's steps are from this one on: the time until its next step. The runtime doesn't blend
    /// across a change of it.
    pub period: Duration,
}

impl Step {
    /// How far to draw from the pose before this step's to this one's, with the clock at `now`: the time since the
    /// step in periods (0 at `at`, 1 a period later, past 1 while the next step is late; 0 before `at`; 1 with no
    /// period). The runtime keeps it within 0..1.
    pub fn alpha(&self, now: Duration) -> f32 {
        if self.period.is_zero() {
            return 1.0;
        }
        now.saturating_sub(self.at).as_secs_f32() / self.period.as_secs_f32()
    }
}

/// A body group's name, the model's engine's (a mode's module names those it knows).
pub type GroupId = u32;

/// A body of a model that can move apart from the rest: its joints, and how it casts its contact shadow.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyGroup {
    pub id: GroupId,
    pub joints: Vec<u16>,
    pub cast: Cast,
}

/// How a body casts its contact shadow on the surface under it: soft ellipses, darker the nearer it is.
#[derive(Clone, Debug, PartialEq)]
pub enum Cast {
    /// One ellipse a bone: from each of its joints to its parent, when the parent is in the group too (the model's
    /// `parents`; none without them). Needs joints whose matrices place them: a glTF source's (a joint's origin).
    Segments,
    /// A fixed shape: `ellipses` laid along the line from `back` to `front` (their middle, level), whatever its
    /// joints do.
    Footprint { back: Point, front: Point, ellipses: Vec<Ellipse> },
}

/// A point carried by a joint: `at` in that joint's matrix's space (the [`Pose`]'s root times the joint's matrix times
/// `at` is where it is). For a glTF source, the zero point is the joint itself; for meshes as data, it is a point of
/// the meshes as given.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub joint: u16,
    pub at: Vec3,
}

/// A soft ellipse of a [`Cast::Footprint`], in units of how tall the model is drawn ([`crate::Focus`]'s `height`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ellipse {
    /// Its middle's distance from the footprint's middle, along it (towards `front`).
    pub along: f32,
    pub half_length: f32,
    pub half_width: f32,
    /// How dark it is right under (0..1).
    pub dark: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_is_carried_by_its_joint_into_the_world() {
        let pose = Pose {
            root: Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0)),
            joints: vec![Mat4::IDENTITY, Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2)],
            ..Default::default()
        };
        let p = pose.point(Point { joint: 1, at: Vec3::X }).unwrap();
        assert!((p - Vec3::new(10.0, 0.0, -1.0)).length() < 1e-6, "{p}");
        assert_eq!(pose.point(Point { joint: 2, at: Vec3::ZERO }), None);
    }

    /// The time since the step, in periods: the same sum as a fixed clock's leftover over its step.
    #[test]
    fn alpha_is_the_time_since_the_step_in_periods() {
        let step = Step { at: Duration::from_millis(100), period: Duration::from_millis(20) };
        assert_eq!(step.alpha(Duration::from_millis(100)), 0.0);
        assert_eq!(step.alpha(Duration::from_millis(105)), 0.25);
        assert_eq!(step.alpha(Duration::from_millis(130)), 1.5);
        assert_eq!(step.alpha(Duration::from_millis(90)), 0.0);
        let leftover = Duration::from_nanos(7_123_457);
        let period = Duration::from_nanos(16_666_667);
        assert_eq!(
            Step { at: Duration::from_secs(3), period }.alpha(Duration::from_secs(3) + leftover).to_bits(),
            (leftover.as_secs_f32() / period.as_secs_f32()).to_bits()
        );
        assert_eq!(Step::default().alpha(Duration::from_secs(1)), 1.0);
    }
}
