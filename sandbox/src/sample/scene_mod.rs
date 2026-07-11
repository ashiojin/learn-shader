use bevy::prelude::*;

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
}

impl TrailEmitter {
    pub fn new(lifetime: f32) -> Self {
        Self {
            lifetime,
            timing: None,
        }
    }

    pub fn with_timing(mut self, timing: TrailEmitterTiming) -> Self {
        self.timing = Some(timing);
        self
    }

    pub fn lifetime(&self) -> f32 {
        self.lifetime
    }

    pub fn timing(&self) -> Option<TrailEmitterTiming> {
        self.timing
    }
}

#[derive(Component, Debug, Clone)]
pub struct CurrentTrailPositions {
    pub begin: Vec3,
    pub end: Vec3,
}
impl CurrentTrailPositions {
    pub fn begin(&self) -> Vec3 {
        self.begin
    }

    pub fn end(&self) -> Vec3 {
        self.end
    }
}
#[derive(Component, Debug, Clone)]
pub struct PreviousTrailPositions {
    pub begin: Vec3,
    pub end: Vec3,
}
impl PreviousTrailPositions {
    pub fn begin(&self) -> Vec3 {
        self.begin
    }

    pub fn end(&self) -> Vec3 {
        self.end
    }
}

pub fn draw_gizmo_for_trail_meshes(
    mut gizmos: Gizmos,
    other_state: Res<crate::config::ConfigState>,
    q_trail_positions: Query<(&CurrentTrailPositions, &PreviousTrailPositions)>,
) {
    if !other_state.enable_gizmos_for_debug() {
        return;
    }
    for (current_trail_positions, previous_trail_positions) in q_trail_positions.iter() {
        let current_begin = current_trail_positions.begin();
        let current_end = current_trail_positions.end();
        let previous_begin = previous_trail_positions.begin();
        let previous_end = previous_trail_positions.end();

        gizmos.arrow(current_begin, current_end, bevy::color::palettes::css::RED);
        gizmos.arrow(previous_begin, previous_end, bevy::color::palettes::css::GREEN);
    }
}
