use crate::{
    Input,
    ui::{Ellipse, Font, Line, Quad, Ui},
};
use cgmath::ElementWise;
use math::{NoE2Rotor, Rotor, Transform};
use std::{collections::HashMap, f32::consts::TAU};
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
}

pub struct State {
    surface_width: u32,
    surface_height: u32,

    camera: Camera,

    space_mono: Font,
    ui: Ui,

    frame_times: [f32; 128],
}

impl State {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self {
            surface_width: 0,
            surface_height: 0,

            camera: Camera::default(),

            space_mono: Font::from_raw(
                device,
                queue,
                include_str!("../fonts/space_mono.fnt"),
                &HashMap::from([
                    (0, include_bytes!("../fonts/space_mono_0.png").as_slice()),
                    (1, include_bytes!("../fonts/space_mono_1.png").as_slice()),
                ]),
            ),
            ui: Ui::new(device, queue),

            frame_times: [0.0; _],
        }
    }

    pub fn update(&mut self, input: &Input, ts: f32) {
        self.frame_times.rotate_right(1);
        self.frame_times[0] = 1.0 / ts;

        // camera stuff
        {
            let speed = 2.0;

            let forward = self.camera.rotation.x();
            let up = self.camera.rotation.y();
            let right = self.camera.rotation.z();
            let ana = self.camera.rotation.w();

            if input.key_pressed(KeyCode::KeyW) {
                self.camera.position += forward * speed * ts;
            }
            if input.key_pressed(KeyCode::KeyS) {
                self.camera.position -= forward * speed * ts;
            }
            if input.key_pressed(KeyCode::KeyA) {
                self.camera.position -= right * speed * ts;
            }
            if input.key_pressed(KeyCode::KeyD) {
                self.camera.position += right * speed * ts;
            }
            if input.key_pressed(KeyCode::KeyQ) {
                self.camera.position -= up * speed * ts;
            }
            if input.key_pressed(KeyCode::KeyE) {
                self.camera.position += up * speed * ts;
            }
            if input.key_pressed(KeyCode::KeyR) {
                self.camera.position += ana * speed * ts;
            }
            if input.key_pressed(KeyCode::KeyF) {
                self.camera.position -= ana * speed * ts;
            }
        }
    }

    pub fn surface_resized(&mut self, width: u32, height: u32) {
        self.surface_width = width;
        self.surface_height = height;
    }

    pub fn mouse_moved(&mut self, input: &Input, old_position: cgmath::Vector2<f32>) {
        let delta = input.mouse_position - old_position;

        let sensitivity = 3.0;

        if input.mouse_button_pressed(MouseButton::Left) {
            self.camera.rotation = self
                .camera
                .rotation
                .then(NoE2Rotor::rotate_xz(delta.x * sensitivity));
            self.camera.xy_rotation += delta.y * sensitivity;
            self.camera.xy_rotation = self.camera.xy_rotation.clamp(-TAU * 0.25, TAU * 0.25);
        }

        if input.mouse_button_pressed(MouseButton::Right) {
            self.camera.rotation = self
                .camera
                .rotation
                .then(NoE2Rotor::rotate_zw(delta.x * sensitivity))
                .then(NoE2Rotor::rotate_xw(delta.y * sensitivity));
        }
    }

    pub fn render<'a>(
        &'a mut self,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        #[expect(unused)] encoder: &mut wgpu::CommandEncoder,
    ) -> impl FnOnce(&mut wgpu::RenderPass<'_>) + use<'a> {
        let aspect = self.surface_width as f32 / self.surface_height as f32;

        self.ui.clear();
        self.ui.push_quad(
            Quad {
                position: cgmath::vec2(0.0, 0.0),
                size: cgmath::vec2(2.0 * aspect, 2.0),
                color: cgmath::vec4(0.0, 0.0, 0.0, 1.0),
            },
            None,
        );

        {
            let compass_size = cgmath::vec2(0.5, 0.5);
            let inner_compass_size = cgmath::vec2(0.45, 0.45);
            let compass_position = cgmath::vec2(1.0 * aspect, 1.0) - compass_size * 0.5;

            self.ui.push_ellipse(
                Ellipse {
                    position: compass_position,
                    size: compass_size,
                    color: cgmath::vec4(1.0, 1.0, 1.0, 0.7),
                },
                None,
            );

            #[rustfmt::skip]
            let mut directions: [(cgmath::Vector4<f32>, cgmath::Vector3<f32>, &str); _] = [
                (cgmath::vec4( 1.0, 0.0,  0.0,  0.0), cgmath::vec3(1.0, 0.0, 0.0), "+X"),
                (cgmath::vec4(-1.0, 0.0,  0.0,  0.0), cgmath::vec3(1.0, 0.0, 0.0), "-X"),
                (cgmath::vec4( 0.0, 0.0,  1.0,  0.0), cgmath::vec3(0.0, 0.0, 1.0), "+Z"),
                (cgmath::vec4( 0.0, 0.0, -1.0,  0.0), cgmath::vec3(0.0, 0.0, 1.0), "-Z"),
                (cgmath::vec4( 0.0, 0.0,  0.0,  1.0), cgmath::vec3(1.0, 0.0, 1.0), "+W"),
                (cgmath::vec4( 0.0, 0.0,  0.0, -1.0), cgmath::vec3(1.0, 0.0, 1.0), "-W"),
            ];
            for (direction, _, _) in &mut directions {
                *direction = self
                    .camera
                    .rotation
                    .reverse()
                    .transform_direction(*direction);
            }
            directions.sort_by(|(a, _, _), (b, _, _)| a.w.total_cmp(&b.w));
            for (direction, color, name) in directions {
                self.ui.push_line(Line {
                    a: compass_position,
                    b: compass_position
                        + cgmath::vec2(direction.z, direction.x)
                            .mul_element_wise(inner_compass_size * 0.5),
                    color,
                    width: 0.05,
                });

                self.space_mono.draw_str(
                    &mut self.ui,
                    name,
                    compass_position
                        + cgmath::vec2(direction.z, direction.x)
                            .mul_element_wise(inner_compass_size * 0.45),
                    0.1,
                    cgmath::vec4(0.0, 0.0, 0.0, 1.0),
                );
            }
        }

        {
            let fps = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            self.space_mono.draw_str(
                &mut self.ui,
                &format!("FPS: {fps:.2}"),
                cgmath::vec2(0.0, 0.95),
                0.1,
                cgmath::vec4(1.0, 1.0, 1.0, 1.0),
            );
        }

        move |render_pass: &mut wgpu::RenderPass<'_>| {
            self.ui.render(
                device,
                queue,
                render_pass,
                self.surface_width,
                self.surface_height,
            );
        }
    }
}

pub(crate) fn render_pipeline(
    device: &wgpu::Device,
    name: &str,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    topology: wgpu::PrimitiveTopology,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(name),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState {
            topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fragment"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
}
