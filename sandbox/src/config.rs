use bevy::{color::palettes::css, prelude::*};

use crate::sample::SampleModel;

#[derive(Resource, Debug, Default)]
pub struct ConfigState {
    enable_gizmos_for_models: bool,
    enable_gizmos_for_debug: bool,
}

impl ConfigState {
    pub fn enable_gizmos_for_models(&self) -> bool {
        self.enable_gizmos_for_models
    }

    pub fn enable_gizmos_for_debug(&self) -> bool {
        self.enable_gizmos_for_debug
    }

    pub fn toggle_gizmo_cross(&mut self) {
        self.enable_gizmos_for_models = !self.enable_gizmos_for_models;
    }

    pub fn toggle_gizmo_for_debug(&mut self) {
        self.enable_gizmos_for_debug = !self.enable_gizmos_for_debug;
    }
}

pub fn draw_gizmo(
    mut gizmos: Gizmos,
    other_state: Res<ConfigState>,
    sample_model: Query<&Transform, With<SampleModel>>,
) {
    if other_state.enable_gizmos_for_models {
        for transform in sample_model.iter() {
            let pos = transform.translation;
            gizmos.arrow(pos - Vec3::X, pos + Vec3::X, css::RED);
            gizmos.arrow(pos - Vec3::Y, pos + Vec3::Y, css::GREEN);
            gizmos.arrow(pos - Vec3::Z, pos + Vec3::Z, css::BLUE);
        }
    }
}

pub fn draw_xy_grid_gizmo(
    mut gizmos: Gizmos,
    other_state: Res<ConfigState>,
) {
    // write X-Y grid on Z=0
    if !other_state.enable_gizmos_for_debug {
        return;
    }
    let grid_size = 10;
    let grid_spacing = 0.5;
    let color = css::GRAY.with_alpha(0.5);
    for i in -grid_size..=grid_size {
        let offset = i as f32 * grid_spacing;
        gizmos.line(
            Vec3::new(-grid_size as f32 * grid_spacing, offset, 0.0),
            Vec3::new(grid_size as f32 * grid_spacing, offset, 0.0),
            color,
        );
        gizmos.line(
            Vec3::new(offset, -grid_size as f32 * grid_spacing, 0.0),
            Vec3::new(offset, grid_size as f32 * grid_spacing, 0.0),
            color,
        );
    }


}
