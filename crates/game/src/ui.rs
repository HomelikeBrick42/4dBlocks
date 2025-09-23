use std::num::NonZeroU64;

use crate::state::render_pipeline;
use bytemuck::{Pod, Zeroable};

pub struct Ui {
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    lines_buffer: wgpu::Buffer,
    lines_bind_group_layout: wgpu::BindGroupLayout,
    lines_pipeline: wgpu::RenderPipeline,

    quads_buffer: wgpu::Buffer,
    quads_bind_group_layout: wgpu::BindGroupLayout,
    quads_pipeline: wgpu::RenderPipeline,

    ellipses_buffer: wgpu::Buffer,
    ellipses_bind_group_layout: wgpu::BindGroupLayout,
    ellipses_pipeline: wgpu::RenderPipeline,

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

        let quads_buffer = quads_buffer(device, 0);
        let quads_bind_group_layout = quads_bind_group_layout(device);

        let quads_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/quads.wgsl"
        )));
        let quads_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Quads Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &quads_bind_group_layout],
                push_constant_ranges: &[],
            });
        let quads_pipeline = render_pipeline(
            device,
            "Quads Render Pipeline",
            &quads_pipeline_layout,
            &quads_shader,
            wgpu::PrimitiveTopology::TriangleStrip,
        );

        let ellipses_buffer = ellipses_buffer(device, 0);
        let ellipses_bind_group_layout = ellipses_bind_group_layout(device);

        let ellipses_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/ellipses.wgsl"
        )));
        let ellipses_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ellipses Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &ellipses_bind_group_layout],
                push_constant_ranges: &[],
            });
        let ellipses_pipeline = render_pipeline(
            device,
            "Ellipses Render Pipeline",
            &ellipses_pipeline_layout,
            &ellipses_shader,
            wgpu::PrimitiveTopology::TriangleStrip,
        );

        Self {
            camera_buffer,
            camera_bind_group,

            lines_buffer,
            lines_bind_group_layout,
            lines_pipeline,

            quads_buffer,
            quads_bind_group_layout,
            quads_pipeline,

            ellipses_buffer,
            ellipses_bind_group_layout,
            ellipses_pipeline,

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

    pub fn push_quad(&mut self, quad: Quad) {
        let Quad {
            position,
            size,
            color,
        } = quad;
        let gpu_quad = GpuQuad {
            position: position.into(),
            size: size.into(),
            color: color.into(),
        };
        if let Some(Layer::Quads(gpu_quads)) = self.layers.last_mut() {
            gpu_quads.push(gpu_quad);
        } else {
            self.layers.push(Layer::Quads(vec![gpu_quad]));
        }
    }

    pub fn push_ellipse(&mut self, ellipse: Ellipse) {
        let Ellipse {
            position,
            size,
            color,
        } = ellipse;
        let gpu_ellipse = GpuEllipse {
            position: position.into(),
            size: size.into(),
            color: color.into(),
        };
        if let Some(Layer::Ellipses(gpu_ellipses)) = self.layers.last_mut() {
            gpu_ellipses.push(gpu_ellipse);
        } else {
            self.layers.push(Layer::Ellipses(vec![gpu_ellipse]));
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
        let mut required_quads_count = 0;
        let mut required_ellipses_count = 0;
        for layer in &self.layers {
            match layer {
                Layer::Lines(gpu_lines) => {
                    required_lines_count += gpu_lines.len();
                }
                Layer::Quads(gpu_quads) => {
                    required_quads_count += gpu_quads.len();
                }
                Layer::Ellipses(gpu_ellipses) => {
                    required_ellipses_count += gpu_ellipses.len();
                }
            }
        }

        if required_lines_count * size_of::<GpuLine>() > self.lines_buffer.size() as _ {
            self.lines_buffer = lines_buffer(device, required_lines_count);
        }
        if required_quads_count * size_of::<GpuQuad>() > self.quads_buffer.size() as _ {
            self.quads_buffer = quads_buffer(device, required_quads_count);
        }
        if required_ellipses_count * size_of::<GpuEllipse>() > self.ellipses_buffer.size() as _ {
            self.ellipses_buffer = ellipses_buffer(device, required_ellipses_count);
        }

        struct GpuLayer<'a> {
            pipeline: &'a wgpu::RenderPipeline,
            bind_group: wgpu::BindGroup,
            vertex_count: u32,
            instance_count: u32,
        }

        let layers = {
            let mut lines_buffer =
                NonZeroU64::new((required_lines_count * size_of::<GpuLine>()) as _)
                    .and_then(|length| queue.write_buffer_with(&self.lines_buffer, 0, length));
            let mut lines_buffer = lines_buffer.as_deref_mut();

            let mut quads_buffer =
                NonZeroU64::new((required_quads_count * size_of::<GpuQuad>()) as _)
                    .and_then(|length| queue.write_buffer_with(&self.quads_buffer, 0, length));
            let mut quads_buffer = quads_buffer.as_deref_mut();

            let mut ellipses_buffer =
                NonZeroU64::new((required_ellipses_count * size_of::<GpuQuad>()) as _)
                    .and_then(|length| queue.write_buffer_with(&self.ellipses_buffer, 0, length));
            let mut ellipses_buffer = ellipses_buffer.as_deref_mut();

            let mut lines_size_so_far = 0usize;
            let mut quads_size_so_far = 0usize;
            let mut ellipses_size_so_far = 0usize;
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
                            pipeline: &self.lines_pipeline,
                            bind_group,
                            vertex_count: 4,
                            instance_count: gpu_lines.len().try_into().expect(
                                "the number of lines in a layer should be less than u32::MAX",
                            ),
                        }
                    }

                    Layer::Quads(gpu_quads) => {
                        let quads_buffer = quads_buffer.as_deref_mut().unwrap_or_default();

                        let size = size_of_val::<[_]>(gpu_quads);
                        quads_buffer[quads_size_so_far..][..size]
                            .copy_from_slice(bytemuck::cast_slice(gpu_quads));

                        let bind_group = quads_bind_group(
                            device,
                            &self.quads_bind_group_layout,
                            &self.quads_buffer,
                            quads_size_so_far,
                            size,
                        );

                        quads_size_so_far += size;

                        GpuLayer {
                            pipeline: &self.quads_pipeline,
                            bind_group,
                            vertex_count: 4,
                            instance_count: gpu_quads.len().try_into().expect(
                                "the number of quads in a layer should be less than u32::MAX",
                            ),
                        }
                    }

                    Layer::Ellipses(gpu_ellipses) => {
                        let ellipses_buffer = ellipses_buffer.as_deref_mut().unwrap_or_default();

                        let size = size_of_val::<[_]>(gpu_ellipses);
                        ellipses_buffer[ellipses_size_so_far..][..size]
                            .copy_from_slice(bytemuck::cast_slice(gpu_ellipses));

                        let bind_group = ellipses_bind_group(
                            device,
                            &self.ellipses_bind_group_layout,
                            &self.ellipses_buffer,
                            ellipses_size_so_far,
                            size,
                        );

                        ellipses_size_so_far += size;

                        GpuLayer {
                            pipeline: &self.ellipses_pipeline,
                            bind_group,
                            vertex_count: 4,
                            instance_count: gpu_ellipses.len().try_into().expect(
                                "the number of ellipses in a layer should be less than u32::MAX",
                            ),
                        }
                    }
                })
                .collect::<Vec<_>>()
        };
        queue.submit(std::iter::empty());

        for GpuLayer {
            pipeline,
            bind_group,
            vertex_count,
            instance_count,
        } in layers
        {
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &bind_group, &[]);
            render_pass.draw(0..vertex_count, 0..instance_count);
        }
    }
}

pub struct Line {
    pub a: cgmath::Vector2<f32>,
    pub b: cgmath::Vector2<f32>,
    pub color: cgmath::Vector4<f32>,
    pub width: f32,
}

pub struct Quad {
    pub position: cgmath::Vector2<f32>,
    pub size: cgmath::Vector2<f32>,
    pub color: cgmath::Vector4<f32>,
}

pub struct Ellipse {
    pub position: cgmath::Vector2<f32>,
    pub size: cgmath::Vector2<f32>,
    pub color: cgmath::Vector4<f32>,
}

enum Layer {
    Lines(Vec<GpuLine>),
    Quads(Vec<GpuQuad>),
    Ellipses(Vec<GpuEllipse>),
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
    pub color: [f32; 4],
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

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct GpuQuad {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
}

fn quads_buffer(device: &wgpu::Device, length: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Quads Buffer"),
        size: (length.max(1) * size_of::<GpuQuad>())
            .try_into()
            .expect("the size of the quads buffer should fit in a wgpu::BufferAddress"),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn quads_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Quads Bind Group Layout"),
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

fn quads_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    quads_buffer: &wgpu::Buffer,
    offset: usize,
    size: usize,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Quads Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: quads_buffer,
                offset: offset as _,
                size: NonZeroU64::new(size as _),
            }),
        }],
    })
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct GpuEllipse {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
}

fn ellipses_buffer(device: &wgpu::Device, length: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Ellipses Buffer"),
        size: (length.max(1) * size_of::<GpuEllipse>())
            .try_into()
            .expect("the size of the ellipses buffer should fit in a wgpu::BufferAddress"),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn ellipses_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Ellipses Bind Group Layout"),
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

fn ellipses_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    ellipses_buffer: &wgpu::Buffer,
    offset: usize,
    size: usize,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Ellipses Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: ellipses_buffer,
                offset: offset as _,
                size: NonZeroU64::new(size as _),
            }),
        }],
    })
}
