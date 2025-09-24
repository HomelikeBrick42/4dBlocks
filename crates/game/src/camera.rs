use crate::Input;
use math::{NoE2Rotor, Rotor, Transform};
use std::f32::consts::TAU;
use winit::{event::MouseButton, keyboard::KeyCode};

pub struct Camera {
    pub position: cgmath::Vector4<f32>,
    pub rotation: NoE2Rotor,
    pub xy_rotation: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: cgmath::vec4(0.0, 0.0, 0.0, 0.0),
            rotation: NoE2Rotor::identity(),
            xy_rotation: 0.0,
        }
    }
}

impl Camera {
    pub fn transform(&self) -> Transform {
        Transform::translation(self.position).then(Transform::from_rotor(
            Rotor::from_no_e2_rotor(self.rotation).then(Rotor::rotate_xy(self.xy_rotation)),
        ))
    }

    pub fn update(&mut self, input: &Input, ts: f32) {
        let speed = 2.0;

        let forward = self.rotation.x();
        let up = self.rotation.y();
        let right = self.rotation.z();
        let ana = self.rotation.w();

        if input.key_pressed(KeyCode::KeyW) {
            self.position += forward * speed * ts;
        }
        if input.key_pressed(KeyCode::KeyS) {
            self.position -= forward * speed * ts;
        }
        if input.key_pressed(KeyCode::KeyA) {
            self.position -= right * speed * ts;
        }
        if input.key_pressed(KeyCode::KeyD) {
            self.position += right * speed * ts;
        }
        if input.key_pressed(KeyCode::KeyQ) {
            self.position -= up * speed * ts;
        }
        if input.key_pressed(KeyCode::KeyE) {
            self.position += up * speed * ts;
        }
        if input.key_pressed(KeyCode::KeyR) {
            self.position += ana * speed * ts;
        }
        if input.key_pressed(KeyCode::KeyF) {
            self.position -= ana * speed * ts;
        }
    }

    pub fn mouse_moved(&mut self, input: &Input, delta: cgmath::Vector2<f32>) {
        let sensitivity = 3.0;

        if input.mouse_button_pressed(MouseButton::Left) {
            self.rotation = self
                .rotation
                .then(NoE2Rotor::rotate_xz(delta.x * sensitivity));
            self.xy_rotation += delta.y * sensitivity;
            self.xy_rotation = self.xy_rotation.clamp(-TAU * 0.25, TAU * 0.25);
        }

        if input.mouse_button_pressed(MouseButton::Right) {
            self.rotation = self
                .rotation
                .then(NoE2Rotor::rotate_zw(delta.x * sensitivity))
                .then(NoE2Rotor::rotate_xw(delta.y * sensitivity));
        }
    }
}
