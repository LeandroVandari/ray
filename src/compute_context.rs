use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferBinding, BufferUsages, ComputePipeline,
    ComputePipelineDescriptor, Device, Extent3d, PipelineCompilationOptions,
    PipelineLayoutDescriptor, ShaderModuleDescriptor, Texture, TextureDescriptor, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::objects;

pub struct ComputeContext {
    pub(crate) compute_pipeline: ComputePipeline,
    pub(crate) output_texture: Texture,
    pub(crate) bind_group: BindGroup,
}

impl ComputeContext {
    pub fn new(
        device: &Device,
        output_size: (u32, u32),
        output_format: TextureFormat,
        spheres: &[objects::Sphere],
    ) -> Self {
        let output_texture = Self::create_texture(
            device,
            Extent3d {
                width: output_size.0,
                height: output_size.1,
                depth_or_array_layers: 1,
            },
            output_format,
        );
        let texture_view = output_texture.create_view(&TextureViewDescriptor::default());

        let sphere_buffer = Self::create_sphere_buffer(device, spheres);

        let bind_group_layout = Self::create_bind_group_layout(device, output_format);
        let bind_group =
            Self::create_bind_group(device, &bind_group_layout, &texture_view, &sphere_buffer);

        let compute_pipeline = Self::create_compute_pipeline(device, &bind_group_layout);

        Self {
            compute_pipeline,
            output_texture,
            bind_group,
        }
    }

    fn create_texture(device: &Device, size: Extent3d, format: TextureFormat) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some("Compute Output"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn create_bind_group_layout(device: &Device, format: TextureFormat) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Compute BindGroupLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        texture_view: &TextureView,
        sphere_buffer: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Compute BindGroup"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: sphere_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        })
    }

    fn create_compute_pipeline(
        device: &Device,
        bind_group_layout: &BindGroupLayout,
    ) -> ComputePipeline {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(
                concat!(
                    include_str!("shaders/sphere.wgsl"),
                    include_str!("shaders/ray.wgsl"),
                    include_str!("shaders/hit_record.wgsl"),
                    include_str!("shaders/camera.wgsl"),
                    include_str!("shaders/main.wgsl")
                )
                .into(),
            ),
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: Some("main_compute"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        })
    }

    fn create_sphere_buffer(device: &Device, spheres: &[objects::Sphere]) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Objects Buffer"),
            usage: BufferUsages::STORAGE,
            contents: bytemuck::cast_slice(spheres),
        })
    }
}
