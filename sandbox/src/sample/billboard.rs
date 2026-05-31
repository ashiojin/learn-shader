use bevy::prelude::*;

use crate::{
    camera::SatelliteCamera,
    sample::{SampleModel, SampleState},
};

#[derive(Component, Debug, Clone, Copy)]
pub struct Billboarded {
    camera: Entity,
    model_up: Dir3, // Model's up direction that sync with the up direction of the camera
    model_front: Dir3, // Model's front direction that face to the camera
}

impl Billboarded {
    pub fn new(camera: Entity, model_up: Dir3, model_front: Dir3) -> Self {
        assert!(
            model_up.dot(model_front.as_vec3()).abs() < 1e-6,
            "model_up and model_front should make a right angle to each other"
        );
        Self {
            camera,
            model_up,
            model_front,
        }
    }
}

pub fn update_billboard_transform(
    mut query: Query<(&Billboarded, &mut Transform), Without<Camera>>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    for (billboarded, mut transform) in query.iter_mut() {
        if let Ok(camera_transform) = camera_query.get(billboarded.camera) {
            let camera_pos = camera_transform.translation;
            let entity_pos = transform.translation;

            // Direction from entity to camera
            let forward = (camera_pos - entity_pos).normalize_or_zero();
            if forward == Vec3::ZERO {
                continue;
            }

            let camera_up = camera_transform.up();

            // Target orientation: Z faces camera, Y aligns with camera up
            let target_right = camera_up.cross(forward).normalize_or_zero();
            if target_right == Vec3::ZERO {
                // Fallback if camera_up and forward are parallel (looking straight up/down)
                continue;
            }
            let target_up = forward.cross(target_right);
            let target_rotation =
                Quat::from_mat3(&Mat3::from_cols(target_right, target_up, forward));

            // Model's local orientation basis
            let model_front = billboarded.model_front.as_vec3();
            let model_up = billboarded.model_up.as_vec3();
            let model_right = model_up.cross(model_front).normalize_or_zero();
            if model_right == Vec3::ZERO {
                continue;
            }
            let model_rotation =
                Quat::from_mat3(&Mat3::from_cols(model_right, model_up, model_front));

            // Combine rotations: transform.rotation * model_rotation = target_rotation
            transform.rotation = target_rotation * model_rotation.inverse();
        }
    }
}

pub fn add_billboard_component(
    mut commands: Commands,
    sample_state: Res<SampleState>,
    q_camera: Single<Entity, With<SatelliteCamera>>,
    q_models: Query<Entity, Added<SampleModel>>,
) {
    if sample_state.model_billboard {
        let camera_entity = *q_camera;
        for entity in q_models.iter() {
            commands
                .entity(entity)
                .insert(Billboarded::new(camera_entity, Dir3::Y, Dir3::Z));
        }
    }

    // When SampleState changes, we will refresh Emitters and Models, so we don't need to worry about removing the Billboarded component when switching to a sample that doesn't use it.
}
