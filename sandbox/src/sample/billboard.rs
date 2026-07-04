use bevy::prelude::*;

use crate::{
    billboard::Billboarded,
    camera::SatelliteCamera,
    sample::{SampleModel, SampleState},
};

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
                .try_insert(Billboarded::new(camera_entity, Dir3::Y, Dir3::Z));
        }
    }

    // When SampleState changes, we will refresh Emitters and Models, so we don't need to worry about removing the Billboarded component when switching to a sample that doesn't use it.
}
