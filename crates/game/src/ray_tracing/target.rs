use crate::ui::Texture;

pub struct RayTracingTarget {
    pub(super) texture: Texture,
    pub(super) write_bind_group: wgpu::BindGroup,
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

        let write_bind_group_layout = write_bind_group_layout(device);
        let write_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{name} Write Bind Group")),
            layout: &write_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture.texture_view()),
            }],
        });

        Self {
            texture,
            write_bind_group,
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}

pub(super) fn write_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture Write Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba32Float,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }],
    })
}
