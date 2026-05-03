use std::f32::consts::PI;

use bevy::prelude::*;

use crate::camera::{FollowCamera, SatelliteCamera};

#[derive(Resource, Debug, Default)]
pub struct LightState {
    pattern: LightPattern,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LightPattern {
    /// Ambient light only
    BrightAmbientLightOnly(BrightAmbientLightOnlySetting),
    /// Balanced Key/Fill/Backlight
    Studio(StudioSetting),
    /// Point light that orbits
    RotatingPoint(RotatingPointSetting),
    /// Single strong directional light
    HighContrast(HighContrastSetting),
    /// Light attached to camera
    Flashlight(FlashlightSetting),
    /// Emissive-only testing
    Dark(DarkSetting),
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PointLightSetting {
    intensity: f32,
    range: f32,
    radius: f32,
    position: Vec3,
}
impl PointLightSetting {
    fn make_point_light(&self) -> PointLight {
        PointLight {
            color: Color::WHITE,
            intensity: self.intensity,
            range: self.range,
            radius: self.radius,
            shadows_enabled: true,
            ..default()
        }
    }
}
impl Default for LightPattern {
    fn default() -> Self {
        Self::BrightAmbientLightOnly(BrightAmbientLightOnlySetting::default())
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BrightAmbientLightOnlySetting {
    brightness: f32,
}
impl Default for BrightAmbientLightOnlySetting {
    fn default() -> Self {
        Self { brightness: 1000.0 }
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StudioSetting {
    key: PointLightSetting,
    fill: PointLightSetting,
    back: PointLightSetting,
}
impl Default for StudioSetting {
    fn default() -> Self {
        Self {
            key: PointLightSetting {
                intensity: 100_000.0,
                range: 10.0,
                radius: 5.0,
                position: Vec3::new(2.0, 2.0, 2.0),
            },
            fill: PointLightSetting {
                intensity: 50_000.0,
                range: 10.0,
                radius: 5.0,
                position: Vec3::new(-2.0, 2.0, 2.0),
            },
            back: PointLightSetting {
                intensity: 30_000.0,
                range: 10.0,
                radius: 5.0,
                position: Vec3::new(0.0, 2.0, -2.0),
            },
        }
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RotatingPointSetting {
    point_light: PointLightSetting,
}
impl Default for RotatingPointSetting {
    fn default() -> Self {
        Self {
            point_light: PointLightSetting {
                intensity: 100_000.0,
                range: 10.0,
                radius: 5.0,
                position: Vec3::new(2.0, 2.0, 2.0),
            },
        }
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct HighContrastSetting {
    illuminance: f32,
}
impl Default for HighContrastSetting {
    fn default() -> Self {
        Self {
            illuminance: 3000.0,
        }
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct FlashlightSetting {
    point_light: PointLightSetting,
}
impl Default for FlashlightSetting {
    fn default() -> Self {
        Self {
            point_light: PointLightSetting {
                intensity: 100_000.0,
                range: 10.0,
                radius: 5.0,
                position: Vec3::ZERO, // will be overridden by FollowCamera
            },
        }
    }
}
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct DarkSetting {}

impl LightPattern {
    fn next(self) -> Self {
        match self {
            LightPattern::BrightAmbientLightOnly(_) => {
                LightPattern::Studio(StudioSetting::default())
            }
            LightPattern::Studio(_) => LightPattern::RotatingPoint(RotatingPointSetting::default()),
            LightPattern::RotatingPoint(_) => {
                LightPattern::HighContrast(HighContrastSetting::default())
            }
            LightPattern::HighContrast(_) => LightPattern::Flashlight(FlashlightSetting::default()),
            LightPattern::Flashlight(_) => LightPattern::Dark(DarkSetting::default()),
            LightPattern::Dark(_) => {
                LightPattern::BrightAmbientLightOnly(BrightAmbientLightOnlySetting::default())
            }
        }
    }
}

impl LightState {
    pub fn next_pattern(&mut self) {
        self.pattern = self.pattern.next();
    }
}

#[derive(Component, Debug)]
pub struct TestLight;

#[derive(Component, Debug)]
pub struct RotateLight;

pub fn change_light(
    mut commands: Commands,
    light_state: Res<LightState>,
    query: Query<(Entity, &TestLight)>,
    q_camera: Query<(Entity, Option<&SatelliteCamera>), With<Camera>>,
) {
    info!("Changing light pattern to: {:?}", light_state.pattern);
    let mut main_camera_entity = Entity::PLACEHOLDER;
    for (entity, _pattern_component) in query.iter() {
        commands.entity(entity).despawn();
    }
    for (camera_entity, satelite) in q_camera.iter() {
        commands.entity(camera_entity).remove::<AmbientLight>();
        if satelite.is_some() {
            main_camera_entity = camera_entity;
        }
    }

    // Add new light component(s) based on the new pattern
    match light_state.pattern {
        LightPattern::BrightAmbientLightOnly(s) => {
            for (camera_entity, _) in q_camera.iter() {
                commands.entity(camera_entity).insert(AmbientLight {
                    color: Color::WHITE,
                    brightness: s.brightness,
                    affects_lightmapped_meshes: true,
                });
            }
        }
        LightPattern::Studio(s) => {
            commands.spawn((
                TestLight,
                s.key.make_point_light(),
                Transform::from_translation(s.key.position),
            ));
            commands.spawn((
                TestLight,
                s.fill.make_point_light(),
                Transform::from_translation(s.fill.position),
            ));
            commands.spawn((
                TestLight,
                s.back.make_point_light(),
                Transform::from_translation(s.back.position),
            ));
        }
        LightPattern::RotatingPoint(p) => {
            commands.spawn((
                TestLight,
                p.point_light.make_point_light(),
                Transform::from_translation(p.point_light.position),
                RotateLight,
            ));
        }
        LightPattern::HighContrast(b) => {
            commands.spawn((
                TestLight,
                DirectionalLight {
                    color: Color::WHITE,
                    illuminance: b.illuminance,
                    shadows_enabled: true,
                    ..default()
                },
                // `DirectionalLight` shines along the forward direction
                Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
            ));
        }
        LightPattern::Flashlight(b) => {
            commands.spawn((
                TestLight,
                b.point_light.make_point_light(),
                Transform::from_translation(b.point_light.position),
                FollowCamera(main_camera_entity),
            ));
        }
        LightPattern::Dark(_s) => {
            // No lights
        }
    }
}

pub fn update_rotate_light(time: Res<Time>, mut query: Query<&mut Transform, With<RotateLight>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(PI * time.delta_secs() / 4.0));
    }
}
