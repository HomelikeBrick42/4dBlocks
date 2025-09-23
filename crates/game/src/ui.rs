use std::num::NonZeroU64;

use crate::state::render_pipeline;
use bytemuck::{Pod, Zeroable};

pub struct Ui {
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    lines_buffer: wgpu::Buffer,
    lines_bind_group_layout: wgpu::BindGroupLayout,
    lines_pipeline: wgpu::RenderPipeline,

    layers: Vec<Layer>,
}

impl Ui {
    pub fn new(device: &wgpu::Device) -> Self {
        let camera_buffer = camera_buffer(device);
        let camera_bind_group_layout = camera_bind_group_layout(device);
        let camera_bind_group =
            camera_bind_group(device, &camera_bind_group_layout, &camera_buffer);

        let lines_buffer = lines_buffer(device, 0);
        let lines_bind_group_layout = lines_bind_group_layout(device);

        let lines_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/lines.wgsl"
        )));
        let lines_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Lines Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &lines_bind_group_layout],
                push_constant_ranges: &[],
            });
        let lines_pipeline = render_pipeline(
            device,
            "Lines Render Pipeline",
            &lines_pipeline_layout,
            &lines_shader,
            wgpu::PrimitiveTopology::TriangleStrip,
        );

        Self {
            camera_buffer,
            camera_bind_group,

            lines_buffer,
            lines_bind_group_layout,
            lines_pipeline,

            layers: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.layers.clear();
    }

    pub fn push_line(&mut self, line: Line) {
        let Line { a, b, color, width } = line;
        let gpu_line = GpuLine {
            a: a.into(),
            b: b.into(),
            color: color.into(),
            width,
        };
        if let Some(Layer::Lines(gpu_lines)) = self.layers.last_mut() {
            gpu_lines.push(gpu_line);
        } else {
            self.layers.push(Layer::Lines(vec![gpu_line]));
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        width: u32,
        height: u32,
    ) {
        {
            let gpu_camera = GpuCamera {
                aspect: width as f32 / height as f32,
            };
            queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&gpu_camera));
        }

        let mut required_lines_count = 0;
        for layer in &self.layers {
            match layer {
                Layer::Lines(gpu_lines) => {
                    required_lines_count += gpu_lines.len();
                }
            }
        }

        if required_lines_count * size_of::<GpuLine>() > self.lines_buffer.size() as _ {
            self.lines_buffer = lines_buffer(device, required_lines_count);
        }

        struct GpuLayer {
            bind_group: wgpu::BindGroup,
            vertex_count: u32,
            instance_count: u32,
        }

        let layers = {
            let mut lines_buffer =
                NonZeroU64::new((required_lines_count * size_of::<GpuLine>()) as _)
                    .and_then(|length| queue.write_buffer_with(&self.lines_buffer, 0, length));
            let mut lines_buffer = lines_buffer.as_deref_mut();

            let mut lines_size_so_far = 0usize;
            self.layers
                .iter()
                .map(|layer| match layer {
                    Layer::Lines(gpu_lines) => {
                        let lines_buffer = lines_buffer.as_deref_mut().unwrap_or_default();

                        let size = size_of_val::<[_]>(gpu_lines);
                        lines_buffer[lines_size_so_far..][..size]
                            .copy_from_slice(bytemuck::cast_slice(gpu_lines));

                        let bind_group = lines_bind_group(
                            device,
                            &self.lines_bind_group_layout,
                            &self.lines_buffer,
                            lines_size_so_far,
                            size,
                        );

                        lines_size_so_far += size;

                        GpuLayer {
                            bind_group,
                            vertex_count: 4,
                            instance_count: gpu_lines.len().try_into().expect(
                                "the number of lines in a layer should be less than u32::MAX",
                            ),
                        }
                    }
                })
                .collect::<Vec<_>>()
        };

        render_pass.set_pipeline(&self.lines_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        for GpuLayer {
            bind_group,
            vertex_count,
            instance_count,
        } in layers
        {
            render_pass.set_bind_group(1, &bind_group, &[]);
            render_pass.draw(0..vertex_count, 0..instance_count);
        }
    }
}

pub struct Line {
    pub a: cgmath::Vector2<f32>,
    pub b: cgmath::Vector2<f32>,
    pub color: cgmath::Vector3<f32>,
    pub width: f32,
}

enum Layer {
    Lines(Vec<GpuLine>),
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct GpuCamera {
    pub aspect: f32,
}

fn camera_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Camera Buffer"),
        size: size_of::<GpuCamera>().next_multiple_of(16) as _,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn camera_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    camera_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Camera Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
    })
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct GpuLine {
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub color: [f32; 3],
    pub width: f32,
}

fn lines_buffer(device: &wgpu::Device, length: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Lines Buffer"),
        size: (length.max(1) * size_of::<GpuLine>())
            .try_into()
            .expect("the size of the lines buffer should fit in a wgpu::BufferAddress"),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn lines_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Lines Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn lines_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    lines_buffer: &wgpu::Buffer,
    offset: usize,
    size: usize,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Lines Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: lines_buffer,
                offset: offset as _,
                size: NonZeroU64::new(size as _),
            }),
        }],
    })
}
