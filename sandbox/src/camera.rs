use std::f32::consts::PI;

use bevy::{ecs::component::Component, math::{Quat, Vec3}, transform::components::Transform};


#[derive(Component, Debug)]
pub struct SatelliteCamera {
    rotate_y: f32,
    rotate_x: f32,
    distance: f32,

    default_distance: f32,

    center: Vec3,
    up: Vec3,

    /// The speed of camera rotation, in radians per second.
    rotate_speed: f32,

    /// The speed of camera zoom, in units per second.
    zoom_speed: f32,
}
pub enum RotateDirection {
    Up,
    Down,
    Left,
    Right,
}
pub enum ZoomDirection {
    In,
    Out,
}
impl SatelliteCamera {
    pub fn new(distance: f32) -> Self {
        Self {
            rotate_y: 0.0,
            rotate_x: 0.0,
            distance,
            default_distance: distance,
            center: Vec3::ZERO,
            up: Vec3::Y,
            rotate_speed: PI,
            zoom_speed: distance / 2.0,
        }
    }

    pub fn make_transform(&self) -> Transform {
        let mut t = Transform::from_xyz(0.0, 0.0, self.distance);

        // rotate x around center
        t.rotate_around(self.center, Quat::from_rotation_x(self.rotate_x));
        // rotate y around center
        t.rotate_around(self.center, Quat::from_rotation_y(self.rotate_y));

        t.looking_at(self.center, self.up)
    }

    pub fn reset(&mut self) {
        self.rotate_y = 0.0;
        self.rotate_x = 0.0;
        self.distance = self.default_distance;
    }

    pub fn rotate(&mut self, direction: RotateDirection, delta_time: f32) {
        let delta = self.rotate_speed * delta_time;
        match direction {
            RotateDirection::Up => self.add_rotate_x(delta),
            RotateDirection::Down => self.add_rotate_x(-delta),
            RotateDirection::Left => self.add_rotate_y(delta),
            RotateDirection::Right => self.add_rotate_y(-delta),
        }
    }

    pub fn zoom(&mut self, direction: ZoomDirection, delt_time: f32) {
        let delta = self.zoom_speed * delt_time * match direction {
            ZoomDirection::In => -1.0,
            ZoomDirection::Out => 1.0,
        };
        self.distance += delta;
        if self.distance < 0.1 {
            self.distance = 0.1;
        }
    }

    fn add_rotate_y(&mut self, delta: f32) {
        self.rotate_y += delta;
        // keep rotate_y in range [0, 2PI]
        if self.rotate_y > 2.0 * PI {
            self.rotate_y -= 2.0 * PI;
        } else if self.rotate_y < 0.0 {
            self.rotate_y += 2.0 * PI;
        }
    }

    fn add_rotate_x(&mut self, delta: f32) {
        self.rotate_x += delta;
        let ep = 0.01;
        // keep rotate_x in range [-PI/2 + ep, PI/2 - ep]
        if self.rotate_x > PI / 2.0 - ep {
            self.rotate_x = PI / 2.0 - ep;
        } else if self.rotate_x < -PI / 2.0 + ep {
            self.rotate_x = -PI / 2.0 + ep;
        }
    }
}
