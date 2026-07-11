use bevy::prelude::*;
use std::collections::VecDeque;
use my_meshes::{SplineTrailPoint, TrailInterpolationMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationType {
    #[default]
    Repeat,
}

#[derive(Component, Debug, Clone)]
pub struct AutoAnimation {
    clip_index: usize,
    animation_type: AnimationType,
}

impl AutoAnimation {
    pub fn new(index: usize, animation_type: AnimationType) -> Self {
        Self {
            clip_index: index,
            animation_type,
        }
    }

    pub fn clip_index(&self) -> usize {
        self.clip_index
    }

    pub fn animation_type(&self) -> AnimationType {
        self.animation_type
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TrailEmitterTiming {
    pub start_time: f32,
    pub end_time: f32,
}
impl TrailEmitterTiming {
    pub fn new(start_time: f32, end_time: f32) -> Self {
        Self {
            start_time,
            end_time,
        }
    }

    pub fn is_active(&self, time: f32) -> bool {
        time >= self.start_time && time <= self.end_time
    }
}

#[derive(Component, Debug, Clone)]
pub struct TrailEmitter {
    lifetime: f32,
    timing: Option<TrailEmitterTiming>,
    mode: TrailInterpolationMode,
    subdivisions: u32,
}

#[allow(dead_code)]
impl TrailEmitter {
    pub fn new(lifetime: f32) -> Self {
        Self {
            lifetime,
            timing: None,
            mode: TrailInterpolationMode::LinearLastSegment,
            subdivisions: 8,
        }
    }

    pub fn with_timing(mut self, timing: TrailEmitterTiming) -> Self {
        self.timing = Some(timing);
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

    pub fn timing(&self) -> Option<TrailEmitterTiming> {
        self.timing
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
}

impl TrailHistory {
    pub fn new(mode: TrailInterpolationMode, subdivisions: u32) -> Self {
        Self {
            points: VecDeque::new(),
            trail_entity: None,
            mode,
            subdivisions,
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
            gizmos.line(p1.root, p2.root, bevy::color::palettes::css::RED);
            gizmos.line(p1.tip, p2.tip, bevy::color::palettes::css::GREEN);
            gizmos.line(p1.root, p1.tip, bevy::color::palettes::css::BLUE);
        }
        if let Some(last) = points.back() {
            gizmos.line(last.root, last.tip, bevy::color::palettes::css::YELLOW);
        }
    }
}
