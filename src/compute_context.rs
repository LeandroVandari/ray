use std::sync::{Arc, atomic::AtomicU32};

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferUsages, CommandEncoder, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, PipelineCompilationOptions,
    PipelineLayoutDescriptor, ShaderModuleDescriptor, Texture, TextureDescriptor, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::objects;

#[derive(Debug)]
pub struct ComputeContext {
    pub(crate) compute_pipeline: ComputePipeline,

    pub(crate) previous_texture: Texture,
    pub(crate) output_texture: Texture,
    pub(crate) textures_bind_groups: [BindGroup; 2],

    pub(crate) frame: Arc<AtomicU32>,
    pub(crate) frame_uniform: Buffer,
    pub(crate) settings_bind_group: BindGroup,
}

impl ComputeContext {
    pub fn new(device: &Device, output_size: (u32, u32), spheres: &[objects::Sphere]) -> Self {
        let output_format = TextureFormat::Rgba8Unorm;
        let texture_size = Extent3d {
            width: output_size.0,
            height: output_size.1,
            depth_or_array_layers: 1,
        };
        let output_texture = Self::create_texture(device, texture_size, output_format);
        let output_texture_view = output_texture.create_view(&TextureViewDescriptor::default());

        let previous_texture = Self::create_texture(device, texture_size, output_format);
        let previous_texture_view = previous_texture.create_view(&TextureViewDescriptor::default());

        let textures_bind_group_layout =
            Self::create_textures_bind_group_layout(device, output_format);

        let textures_bind_groups = [
            (&output_texture_view, &previous_texture_view),
            (&previous_texture_view, &output_texture_view),
        ]
        .map(|(write_to, read_from)| {
            Self::create_textures_bind_group(
                device,
                &textures_bind_group_layout,
                write_to,
                read_from,
            )
        });

        let sphere_buffer = Self::create_sphere_buffer(device, spheres);
        let frame_uniform = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Frame Uniform"),
            // Uniform buffers must be aligned to 16 bytes
            contents: &0u128.to_be_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let settings_bind_group_layout = Self::create_settings_layout(device);
        let settings_bind_group = Self::create_settings_bind_group(
            device,
            &settings_bind_group_layout,
            &sphere_buffer,
            &frame_uniform,
        );

        let compute_pipeline = Self::create_compute_pipeline(
            device,
            &textures_bind_group_layout,
            &settings_bind_group_layout,
        );

        Self {
            compute_pipeline,
            output_texture,
            textures_bind_groups,
            previous_texture,
            frame: Arc::new(AtomicU32::new(0)),
            frame_uniform,
            settings_bind_group,
        }
    }

    pub fn draw(&self, encoder: &mut CommandEncoder) {
        let frame = self
            .frame
            .fetch_add(1, std::sync::atomic::Ordering::Release) as usize;
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.textures_bind_groups[frame % 2], &[]);
            compute_pass.set_bind_group(1, &self.settings_bind_group, &[]);
            compute_pass.dispatch_workgroups(
                self.output_texture.width() / 8 + 1,
                self.output_texture.height() / 8 + 1,
                1,
            );
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
            usage: TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn create_textures_bind_group_layout(
        device: &Device,
        format: TextureFormat,
    ) -> BindGroupLayout {
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_textures_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        output_texture_view: &TextureView,
        previous_texture_view: &TextureView,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Compute BindGroup"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(output_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(previous_texture_view),
                },
            ],
        })
    }

    fn create_settings_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Compute BindGroupLayout"),
            entries: &[
                // Spheres
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Frame
                BindGroupLayoutEntry {
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

    fn create_settings_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        sphere_buffer: &Buffer,
        frame_uniform: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Settings"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: sphere_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: frame_uniform.as_entire_binding(),
                },
            ],
        })
    }

    fn create_compute_pipeline(
        device: &Device,
        textures_bind_group_layout: &BindGroupLayout,
        settings_bind_group_layout: &BindGroupLayout,
    ) -> ComputePipeline {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(
                concat!(
                    include_str!("shaders/compute/math.wgsl"),
                    include_str!("shaders/compute/random.wgsl"),
                    include_str!("shaders/compute/interval.wgsl"),
                    include_str!("shaders/compute/sphere.wgsl"),
                    include_str!("shaders/compute/ray.wgsl"),
                    include_str!("shaders/compute/hit_record.wgsl"),
                    include_str!("shaders/compute/material.wgsl"),
                    include_str!("shaders/compute/camera.wgsl"),
                    include_str!("shaders/compute/main.wgsl")
                )
                .into(),
            ),
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[textures_bind_group_layout, settings_bind_group_layout],
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
