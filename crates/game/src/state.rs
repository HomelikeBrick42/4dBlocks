use crate::{
    Input,
    camera::Camera,
    ray_tracing::{CameraBasis, RayTracing, RayTracingTarget},
    ui::{Ellipse, Font, Line, Quad, TextureInfo, Ui},
};
use cgmath::ElementWise;

pub struct State {
    surface_width: u32,
    surface_height: u32,

    camera: Camera,

    space_mono: Font,
    ui: Ui,

    frame_times: [f32; 128],

    ray_tracing: RayTracing,
    main_view: RayTracingTarget,
}

impl State {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let surface_width = 1;
        let surface_height = 1;

        let space_mono = Font::from_raw(
            device,
            queue,
            include_str!("../fonts/space_mono.fnt"),
            &<_>::from([
                (0, include_bytes!("../fonts/space_mono_0.png").as_slice()),
                (1, include_bytes!("../fonts/space_mono_1.png").as_slice()),
            ]),
        );

        let ray_tracing = RayTracing::new(device);
        let main_view =
            RayTracingTarget::new(device, "Main View Texture", surface_width, surface_height);

        Self {
            surface_width,
            surface_height,

            camera: Camera::default(),

            space_mono,
            ui: Ui::new(device, queue),

            frame_times: [0.0; _],

            ray_tracing,
            main_view,
        }
    }

    pub fn update(&mut self, input: &Input, ts: f32) {
        self.frame_times.rotate_right(1);
        self.frame_times[0] = 1.0 / ts;

        self.camera.update(input, ts);
    }

    pub fn surface_resized(&mut self, width: u32, height: u32) {
        self.surface_width = width;
        self.surface_height = height;
    }

    pub fn mouse_moved(&mut self, input: &Input, old_position: cgmath::Vector2<f32>) {
        let delta = input.mouse_position - old_position;
        self.camera.mouse_moved(input, delta);
    }

    pub fn render<'a>(
        &'a mut self,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) -> impl FnOnce(&mut wgpu::RenderPass<'_>) + use<'a> {
        let aspect = self.surface_width as f32 / self.surface_height as f32;

        // render main view
        {
            let main_view_size = self.main_view.texture().texture_view().texture().size();
            if main_view_size.width != self.surface_width
                || main_view_size.height != self.surface_height
            {
                self.main_view = RayTracingTarget::new(
                    device,
                    "Main View Texture",
                    self.surface_width,
                    self.surface_height,
                );
            }
            self.ray_tracing.render(
                queue,
                self.camera.transform(),
                CameraBasis::XYZ,
                &self.main_view,
                encoder,
            );
        }

        self.ui.clear();
        self.ui.push_quad(
            Quad {
                position: cgmath::vec2(0.0, 0.0),
                size: cgmath::vec2(2.0 * aspect, 2.0),
                color: cgmath::vec4(1.0, 1.0, 1.0, 1.0),
            },
            Some(TextureInfo {
                texture: self.main_view.texture().clone(),
                uv_offset: cgmath::vec2(0.0, 0.0),
                uv_size: cgmath::vec2(1.0, 1.0),
            }),
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
