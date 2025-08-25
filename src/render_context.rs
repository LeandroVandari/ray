use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, ColorTargetState, Device, FragmentState, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderStages, TextureView,
    TextureViewDescriptor, VertexState,
};

use crate::{
    compute_context::ComputeContext,
    gpu_manager::{GpuManager, WindowManager},
};

pub struct RenderContext {
    pub(crate) bind_group: BindGroup,
    pub(crate) pipeline: RenderPipeline,
}

impl RenderContext {
    pub fn new(
        device: &Device,
        compute_context: &ComputeContext,
        gpu_manager: &GpuManager<WindowManager<'_>>,
    ) -> Self {
        let bind_group_layout = Self::create_bind_group_layout(device);
        let bind_group = Self::create_bind_group(
            device,
            &compute_context
                .output_texture
                .create_view(&TextureViewDescriptor::default()),
            &bind_group_layout,
        );

        let pipeline = Self::create_render_pipeline(device, &bind_group_layout, gpu_manager);

        Self {
            pipeline,
            bind_group,
        }
    }

    fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Render BindGroupLayout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    multisampled: false,
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
        gpu_manager: &GpuManager<WindowManager<'_>>,
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
                targets: &[Some(ColorTargetState {
                    format: gpu_manager.config().format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            vertex: VertexState {
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
                module: &vertex_shader,
                entry_point: Some("main_vertex"),
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }
}
