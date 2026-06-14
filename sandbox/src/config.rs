use bevy::{color::palettes::css, prelude::*};

use crate::sample::{SampleModel, scene_mod::{CurrentTrailPositions, PreviousTrailPositions}};

#[derive(Resource, Debug, Default)]
pub struct ConfigState {
    enable_gizmos_for_models: bool,
    enable_gizmos_for_debug: bool,
}

impl ConfigState {
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

pub fn draw_gizmo_for_trail_meshes( // FIXME: Module separation is not good,
                                    // `CurrentTrailPositions` and `PreviousTrailPositions` are
                                    // defined `sample` module. So we should put this function in
                                    // `sample` module and enable `sample` module having access to `ConfigState`.
    mut gizmos: Gizmos,
    other_state: Res<ConfigState>,
    q_trail_positions: Query<(&CurrentTrailPositions, &PreviousTrailPositions)>,
) {
    if !other_state.enable_gizmos_for_debug {
        return;
    }
    for (current_trail_positions, previous_trail_positions) in q_trail_positions.iter() {
        let current_begin = current_trail_positions.begin();
        let current_end = current_trail_positions.end();
        let previous_begin = previous_trail_positions.begin();
        let previous_end = previous_trail_positions.end();

        gizmos.arrow(current_begin, current_end, css::RED);
        gizmos.arrow(previous_begin, previous_end, css::GREEN);
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
