use std::f32::consts::TAU;

use crate::{
    Input,
    ui::{Ellipse, Line, Quad, Ui},
};
use cgmath::ElementWise;
use math::{NoE2Rotor, Rotor, Transform};
use winit::event::MouseButton;

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

    ui: Ui,
}

impl State {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self {
            surface_width: 0,
            surface_height: 0,

            camera: Camera::default(),

            ui: Ui::new(device, queue),
        }
    }

    pub fn update(&mut self, #[expect(unused)] ts: f32) {}

    pub fn surface_resized(&mut self, width: u32, height: u32) {
        self.surface_width = width;
        self.surface_height = height;
    }

    pub fn mouse_moved(&mut self, input: &Input, old_position: cgmath::Vector2<f32>) {
        let delta = input.mouse_position - old_position;

        let sensitivity = 3.0;

        if input.mouse_button_pressed(MouseButton::Left) {
            self.camera.rotation =
                NoE2Rotor::rotate_xz(delta.x * sensitivity).then(self.camera.rotation);
            self.camera.xy_rotation += delta.y * sensitivity;
            self.camera.xy_rotation = self.camera.xy_rotation.clamp(-TAU * 0.25, TAU * 0.25);
        }

        if input.mouse_button_pressed(MouseButton::Right) {
            self.camera.rotation = NoE2Rotor::rotate_zw(-delta.x * sensitivity)
                .then(NoE2Rotor::rotate_xw(-delta.y * sensitivity))
                .then(self.camera.rotation);
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

            let mut directions: [(cgmath::Vector3<f32>, cgmath::Vector3<f32>); _] = [
                (cgmath::vec3(-1.0, 0.0, 0.0), cgmath::vec3(1.0, 0.0, 0.0)),
                (cgmath::vec3(1.0, 0.0, 0.0), cgmath::vec3(1.0, 0.0, 0.0)),
                (cgmath::vec3(0.0, -1.0, 0.0), cgmath::vec3(0.0, 0.0, 1.0)),
                (cgmath::vec3(0.0, 1.0, 0.0), cgmath::vec3(0.0, 0.0, 1.0)),
                (cgmath::vec3(0.0, 0.0, -1.0), cgmath::vec3(1.0, 0.0, 1.0)),
                (cgmath::vec3(0.0, 0.0, 1.0), cgmath::vec3(1.0, 0.0, 1.0)),
            ];
            for (direction, _) in &mut directions {
                let new_direction = self.camera.rotation.transform_direction(cgmath::vec4(
                    direction.x,
                    0.0,
                    direction.y,
                    direction.z,
                ));
                *direction = cgmath::vec3(new_direction.x, new_direction.z, new_direction.w);
            }
            directions.sort_by(|(a, _), (b, _)| a.z.total_cmp(&b.z));
            for (direction, color) in directions {
                self.ui.push_line(Line {
                    a: compass_position,
                    b: compass_position
                        + cgmath::vec2(direction.y, direction.x)
                            .mul_element_wise(inner_compass_size * 0.5),
                    color,
                    width: 0.01,
                });
            }
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
