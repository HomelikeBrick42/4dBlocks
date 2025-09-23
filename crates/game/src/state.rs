use crate::ui::{Line, Quad, Ui};

pub struct State {
    surface_width: u32,
    surface_height: u32,

    ui: Ui,
}

impl State {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            surface_width: 0,
            surface_height: 0,

            ui: Ui::new(device),
        }
    }

    pub fn update(&mut self, #[expect(unused)] ts: f32) {}

    pub fn surface_resized(&mut self, width: u32, height: u32) {
        self.surface_width = width;
        self.surface_height = height;
    }

    pub fn render<'a>(
        &'a mut self,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        #[expect(unused)] encoder: &mut wgpu::CommandEncoder,
    ) -> impl FnOnce(&mut wgpu::RenderPass<'_>) + use<'a> {
        let aspect = self.surface_width as f32 / self.surface_height as f32;

        self.ui.clear();
        self.ui.push_quad(Quad {
            position: cgmath::vec2(0.0, 0.0),
            size: cgmath::vec2(2.0 * aspect, 2.0),
            color: cgmath::vec4(0.0, 0.0, 0.0, 1.0),
        });
        self.ui.push_line(Line {
            a: cgmath::vec2(-1.0, -1.0),
            b: cgmath::vec2(1.0, 1.0),
            color: cgmath::vec4(1.0, 0.0, 0.0, 1.0),
            width: 0.1,
        });
        self.ui.push_quad(Quad {
            position: cgmath::vec2(0.0, 0.0),
            size: cgmath::vec2(1.0, 1.0),
            color: cgmath::vec4(0.0, 1.0, 0.0, 0.5),
        });

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
