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

#[derive(Component, Debug, Clone)]
pub struct TrailEmitter {
    trail_lifetime: f32,
}

impl TrailEmitter {
    pub fn new(trail_lifetime: f32) -> Self {
        Self { trail_lifetime }
    }

    pub fn trail_lifetime(&self) -> f32 {
        self.trail_lifetime
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
                commands.entity(entity).insert(PreviousTrailPositions {
                    begin: current_trail_positions.begin,
                    end: current_trail_positions.end,
                });
            }
            commands.entity(entity).insert(CurrentTrailPositions {
                begin: global_positions[0],
                end: global_positions[1],
            });
        }
    }
}
