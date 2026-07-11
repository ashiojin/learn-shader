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
    begin: Vec3,
    end: Vec3,
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
    begin: Vec3,
    end: Vec3,
}
impl PreviousTrailPositions {
    pub fn begin(&self) -> Vec3 {
        self.begin
    }

    pub fn end(&self) -> Vec3 {
        self.end
    }
}

// Resolved 1-frame delay for the trail to appear by using Plan (B):
// - This system is ordered to run after `TransformSystems::Propagate` in `PostUpdate`.
// - `apply_deferred` is chained between updating positions and spawning the trail so changes are visible immediately.
// - The spawned trail's `GlobalTransform` is manually updated with its final transform, since propagation has already run.
pub fn update_trail_emitter_positions(
    _time: Res<Time>,
    mut commands: Commands,
    q_trail_emitter: Query<(
        Entity,
        &TrailEmitter,
        &Mesh3d,
        &GlobalTransform,
        Option<&CurrentTrailPositions>,
    )>,
    meshes: Res<Assets<Mesh>>,
) {
    for (entity, _trail_emitter, mesh, global_transform, current_trail_positions) in
        q_trail_emitter.iter()
    {
        if let Some(mesh) = meshes.get(&mesh.0) {
            // here we assume there are tow vertices
            let vertices = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            // for now we use only 2 vertices, first and last one, to determine the trail positions. We can extend this to use more vertices if needed.
            let vertices = [vertices[0], vertices[vertices.len() - 1]];
            // get global positions
            let global_positions: Vec<Vec3> = vertices
                .iter()
                .map(|v| global_transform.transform_point(Vec3::from(*v)))
                .collect();

            // update current and previous trail positions
            if let Some(current_trail_positions) = current_trail_positions {
                commands.entity(entity).try_insert(PreviousTrailPositions {
                    begin: current_trail_positions.begin,
                    end: current_trail_positions.end,
                });
            }
            commands.entity(entity).try_insert(CurrentTrailPositions {
                begin: global_positions[0],
                end: global_positions[1],
            });
        }
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
