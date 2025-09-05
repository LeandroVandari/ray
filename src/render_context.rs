use std::sync::{Arc, atomic::AtomicU32};

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, ColorTargetState, CommandEncoder, Device, FragmentState,
    MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderStages, TextureFormat, TextureView, TextureViewDescriptor, VertexState,
};

use crate::compute_context::ComputeContext;

#[derive(Debug)]
pub struct RenderContext {
    pub(crate) bind_groups: [BindGroup; 2],
    pub(crate) pipeline: RenderPipeline,
    frame: Arc<AtomicU32>,
}

impl RenderContext {
    pub fn new(
        device: &Device,
        compute_context: &ComputeContext,
        output_format: TextureFormat,
    ) -> Self {
        let bind_group_layout = Self::create_bind_group_layout(device);
        let bind_groups = [
            &compute_context.output_texture,
            &compute_context.previous_texture,
        ]
        .map(|texture| {
            Self::create_bind_group(
                device,
                &texture.create_view(&TextureViewDescriptor::default()),
                &bind_group_layout,
            )
        });

        let pipeline = Self::create_render_pipeline(device, &bind_group_layout, output_format);

        Self {
            pipeline,
            bind_groups,
            frame: compute_context.frame.clone(),
        }
    }

    pub fn draw_to_texture(&self, encoder: &mut CommandEncoder, view: &TextureView) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("RenderPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_bind_group(
            0,
            &self.bind_groups[self.frame.load(std::sync::atomic::Ordering::Acquire) as usize % 2],
            &[],
        );
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..3, 0..3);
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
        output_format: TextureFormat,
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
                    format: output_format,
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
