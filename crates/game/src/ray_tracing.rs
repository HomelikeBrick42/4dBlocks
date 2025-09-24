use math::Transform;

pub mod target;

pub use target::*;

pub enum CameraBasis {
    XYZ,
    XYW,
    XWZ,
}

pub struct RayTracing {
    chunk_bind_group: wgpu::BindGroup,

    ray_tracing_pipeline: wgpu::ComputePipeline,
}

impl RayTracing {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let target_bind_group_layout = target::bind_group_layout(device);

        let chunk_size = 128usize;

        let chunk_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Chunk Buffer"),
            size: (chunk_size.pow(4) * size_of::<u32>()) as _,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let chunk_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Chunk Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let chunk_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Chunk Bind Group"),
            layout: &chunk_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: chunk_buffer.as_entire_binding(),
            }],
        });

        queue.write_buffer(
            &chunk_buffer,
            0,
            bytemuck::cast_slice(
                &std::iter::repeat_with(|| rand::random_range(0.0..=1.0) > 0.99)
                    .take(chunk_size.pow(4))
                    .collect::<Vec<_>>(),
            ),
        );

        let ray_tracing_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/ray_tracing.wgsl"
        )));
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[&target_bind_group_layout, &chunk_bind_group_layout],
                push_constant_ranges: &[],
            });
        let ray_tracing_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Ray Tracing Pipeline"),
                layout: Some(&ray_tracing_pipeline_layout),
                module: &ray_tracing_shader,
                entry_point: Some("trace_rays"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            chunk_bind_group,

            ray_tracing_pipeline,
        }
    }

    pub fn render(
        &self,
        queue: &wgpu::Queue,
        transform: Transform,
        basis: CameraBasis,
        target: &RayTracingTarget,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let size = target.texture().texture_view().texture().size();

        {
            let x = transform.x().into();
            let y = transform.y().into();
            let z = transform.z().into();
            let w = transform.w().into();

            let (forward, up, right) = match basis {
                CameraBasis::XYZ => (x, y, z),
                CameraBasis::XYW => (x, y, w),
                CameraBasis::XWZ => (x, w, z),
            };

            let camera = GpuCamera {
                position: transform.position().into(),
                forward,
                up,
                right,
                aspect: size.width as f32 / size.height as f32,
            };
            queue.write_buffer(&target.camera_buffer, 0, bytemuck::bytes_of(&camera));
        }

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Ray Tracing Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.ray_tracing_pipeline);
        compute_pass.set_bind_group(0, &target.bind_group, &[]);
        compute_pass.set_bind_group(1, &self.chunk_bind_group, &[]);

        compute_pass.dispatch_workgroups(size.width.div_ceil(16), size.height.div_ceil(16), 1);
    }
}
