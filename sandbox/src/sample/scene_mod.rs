use bevy::prelude::*;
use std::collections::VecDeque;
use my_meshes::{SplineTrailPoint, TrailInterpolationMode};

#[derive(Debug, Clone, Copy)]
pub struct TrailEmitterTiming {
    pub node_idx: AnimationNodeIndex,
    pub start_time: f32,
    pub end_time: f32,
}
impl TrailEmitterTiming {
    pub fn new(node_idx: AnimationNodeIndex, start_time: f32, end_time: f32) -> Self {
        Self {
            node_idx,
            start_time,
            end_time,
        }
    }

    pub fn node_idx(&self) -> AnimationNodeIndex {
        self.node_idx
    }

    pub fn is_on_time(&self, time: f32) -> bool {
        time >= self.start_time && time <= self.end_time
    }
}

#[derive(Component, Debug, Clone)]
pub struct TrailEmitter {
    lifetime: f32,
    timing: Vec<TrailEmitterTiming>,
    mode: TrailInterpolationMode,
    subdivisions: u32,
}

#[allow(dead_code)]
impl TrailEmitter {
    pub fn new(lifetime: f32) -> Self {
        Self {
            lifetime,
            timing: vec![],
            mode: TrailInterpolationMode::LinearLastSegment,
            subdivisions: 8,
        }
    }

    pub fn add_timing(mut self, timing: TrailEmitterTiming) -> Self {
        self.timing.push(timing);
        self
    }

    pub fn extend_timings(mut self, timings: Vec<TrailEmitterTiming>) -> Self {
        self.timing.extend(timings);
        self
    }

    pub fn with_mode(mut self, mode: TrailInterpolationMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_subdivisions(mut self, subdivisions: u32) -> Self {
        self.subdivisions = subdivisions;
        self
    }

    pub fn lifetime(&self) -> f32 {
        self.lifetime
    }

    pub fn timings(&self) -> &Vec<TrailEmitterTiming> {
        &self.timing
    }

    pub fn timings_of(&self, node_idx: AnimationNodeIndex) -> Vec<&TrailEmitterTiming> {
        self.timing
            .iter()
            .filter(|t| t.node_idx() == node_idx)
            .collect()
    }

    pub fn mode(&self) -> TrailInterpolationMode {
        self.mode
    }

    pub fn subdivisions(&self) -> u32 {
        self.subdivisions
    }
}

#[derive(Component, Debug, Clone)]
pub struct TrailHistory {
    pub points: VecDeque<SplineTrailPoint>,
    pub trail_entity: Option<Entity>,
    pub mode: TrailInterpolationMode,
    pub subdivisions: u32,
    /// Whether the emitter was emitting on the previous frame. Used to detect
    /// the start of a new burst (idle -> active), which must cut the trail.
    pub was_active: bool,
    /// Set when a new burst begins so the next pushed point is flagged
    /// `break_before`, even if the exact transition frame pushed no point.
    pub pending_break: bool,
}

impl TrailHistory {
    pub fn new(mode: TrailInterpolationMode, subdivisions: u32) -> Self {
        Self {
            points: VecDeque::new(),
            trail_entity: None,
            mode,
            subdivisions,
            was_active: false,
            pending_break: false,
        }
    }
}

pub fn draw_gizmo_for_trail_meshes(
    mut gizmos: Gizmos,
    other_state: Res<crate::config::ConfigState>,
    q_trail_history: Query<&TrailHistory>,
) {
    if !other_state.enable_gizmos_for_debug() {
        return;
    }
    for history in q_trail_history.iter() {
        let points = &history.points;
        if points.len() < 2 {
            continue;
        }
        for i in 0..points.len() - 1 {
            let p1 = &points[i];
            let p2 = &points[i + 1];
            // Don't draw a connecting line across a break: the two points
            // belong to separate bursts.
            if !p2.break_before {
                gizmos.line(p1.root, p2.root, bevy::color::palettes::css::RED);
                gizmos.line(p1.tip, p2.tip, bevy::color::palettes::css::GREEN);
            }
            gizmos.line(p1.root, p1.tip, bevy::color::palettes::css::BLUE);
        }
        if let Some(last) = points.back() {
            gizmos.line(last.root, last.tip, bevy::color::palettes::css::YELLOW);
        }
    }
}
