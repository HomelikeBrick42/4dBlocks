pub mod target;

pub use target::*;

pub struct RayTracing {
    ray_tracing_pipeline: wgpu::ComputePipeline,
}

impl RayTracing {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture_write_bind_group_layout = target::write_bind_group_layout(device);

        let ray_tracing_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/ray_tracing.wgsl"
        )));
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[&texture_write_bind_group_layout],
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

    pub fn render(&self, target: &RayTracingTarget, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Ray Tracing Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.ray_tracing_pipeline);
        compute_pass.set_bind_group(0, &target.write_bind_group, &[]);

        let size = target.texture().texture_view().texture().size();
        compute_pass.dispatch_workgroups(size.width.div_ceil(16), size.height.div_ceil(16), 1);
    }
}
