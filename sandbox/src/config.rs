use bevy::{color::palettes::css, prelude::*};

use crate::sample::SampleMesh;


#[derive(Resource, Debug, Default)]
pub struct ConfigState {
    gizmo_cross: bool,
}

impl ConfigState {
    pub fn toggle_gizmo_cross(&mut self) {
        self.gizmo_cross = !self.gizmo_cross;
    }
}

pub fn draw_gizmo(
    mut gizmos: Gizmos,
    other_state: Res<ConfigState>,
    sample_mesh: Query<&Transform, With<SampleMesh>>,
) {
    if other_state.gizmo_cross {
        for transform in sample_mesh.iter() {
            let pos = transform.translation;
            gizmos.arrow(pos - Vec3::X, pos + Vec3::X, css::RED);
            gizmos.arrow(pos - Vec3::Y, pos + Vec3::Y, css::GREEN);
            gizmos.arrow(pos - Vec3::Z, pos + Vec3::Z, css::BLUE);
        }
    }
}
