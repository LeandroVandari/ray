use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Device, FragmentState, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderStages, TextureFormat, TextureView, TextureViewDescriptor,
    VertexState,
};

use crate::compute_context::ComputeContext;

pub struct RenderContext {
    bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl RenderContext {
    pub fn new(device: &Device, compute_context: ComputeContext) -> Self {
        let bind_group_layout =
            Self::create_bind_group_layout(device, &compute_context.output_texture.format());
        let bind_group = Self::create_bind_group(
            device,
            &compute_context
                .output_texture
                .create_view(&TextureViewDescriptor::default()),
            &bind_group_layout,
        );

        let pipeline = Self::create_render_pipeline(device, &bind_group_layout);

        Self {
            pipeline,
            bind_group,
        }
    }

    fn create_bind_group_layout(
        device: &Device,
        compute_texture_format: &TextureFormat,
    ) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Render BindGroupLayout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    format: *compute_texture_format,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        })
    }

    fn create_bind_group(
        device: &Device,
        compute_texture_view: &TextureView,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Render BindGroup"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(compute_texture_view),
            }],
        })
    }

    fn create_render_pipeline(
        device: &Device,
        bind_group_layout: &BindGroupLayout,
    ) -> RenderPipeline {
        let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/render/fragment.wgsl").into()),
        });
        let vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/render/vertex.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            fragment: Some(FragmentState {
                module: &fragment_shader,
                entry_point: Some("main_fragment"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[],
            }),
            vertex: VertexState {
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
                module: &vertex_shader,
                entry_point: Some("main_vertex"),
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }
}
