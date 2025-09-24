use math::Transform;

pub mod target;

pub use target::*;

pub enum CameraBasis {
    XYZ,
    XYW,
    XWZ,
}

pub struct RayTracing {
    ray_tracing_pipeline: wgpu::ComputePipeline,
}

impl RayTracing {
    pub fn new(device: &wgpu::Device) -> Self {
        let target_bind_group_layout = target::bind_group_layout(device);

        let ray_tracing_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/ray_tracing.wgsl"
        )));
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[&target_bind_group_layout],
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

        compute_pass.dispatch_workgroups(size.width.div_ceil(16), size.height.div_ceil(16), 1);
    }
}
