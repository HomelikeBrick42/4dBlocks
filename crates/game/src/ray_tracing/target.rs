use crate::ui::Texture;
use bytemuck::{Pod, Zeroable};

pub struct RayTracingTarget {
    pub(super) texture: Texture,
    pub(super) camera_buffer: wgpu::Buffer,
    pub(super) bind_group: wgpu::BindGroup,
}

impl RayTracingTarget {
    pub fn new(device: &wgpu::Device, name: &str, width: u32, height: u32) -> Self {
        let texture = Texture::new(
            device,
            name,
            width,
            height,
            wgpu::TextureUsages::STORAGE_BINDING,
            wgpu::FilterMode::Nearest,
        );

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{name} Camera Uniform Buffer")),
            size: size_of::<GpuCamera>().next_multiple_of(16) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{name} Write Bind Group")),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture.texture_view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            texture,
            camera_buffer,
            bind_group,
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub(crate) struct GpuCamera {
    pub(crate) position: [f32; 4],
    pub(crate) forward: [f32; 4],
    pub(crate) up: [f32; 4],
    pub(crate) right: [f32; 4],
    pub(crate) aspect: f32,
}

pub(super) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture Write Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}
