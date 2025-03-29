use std::sync::Arc;

use anyhow::{Result, bail};
use wgpu::{
    Adapter, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, BufferBinding, BufferUsages,
    ComputePipeline, ComputePipelineDescriptor, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, PipelineCompilationOptions, PipelineLayoutDescriptor, Queue,
    RequestAdapterOptions, ShaderStages, Surface, SurfaceConfiguration, Texture, TextureDescriptor,
    TextureUsages, TextureView, TextureViewDescriptor,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::window::{Window, WindowAttributes};

pub struct RenderContext<'window> {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: Surface<'window>,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) compute_pipeline: ComputePipeline,
    pub(crate) texture: Texture,
    pub(crate) bind_group: BindGroup,
}

impl RenderContext<'_> {
    pub async fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Result<Self> {
        let instance = Self::create_instance();

        let window = Arc::new(Self::create_window(event_loop)?);

        let surface = instance.create_surface(window.clone())?;

        let Some(adapter) = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
        else {
            bail!("Couldn't create adapter.")
        };

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::BGRA8UNORM_STORAGE,
                    ..Default::default()
                },
                None,
            )
            .await?;

        let config = Self::create_surface_configuration(&surface, &adapter, &window);
        surface.configure(&device, &config);

        let texture = Self::create_textures(&device, &window);
        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let obj_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Objects Buffer"),
            usage: BufferUsages::STORAGE,
            contents: todo!(),
        });

        let bind_group_layout = Self::create_bind_group_layout(&device);
        let bind_group =
            Self::create_bind_group(&device, &bind_group_layout, &texture_view, &obj_buffer);
        let compute_pipeline = Self::create_pipeline(&device, &bind_group_layout);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            compute_pipeline,
            texture,
            bind_group,
        })
    }

    fn create_instance() -> Instance {
        let instance_desc = InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        };
        Instance::new(&instance_desc)
    }

    fn create_window(
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<Window, winit::error::OsError> {
        event_loop.create_window(
            WindowAttributes::default()
                .with_maximized(true)
                .with_resizable(false)
                .with_title("Ray tracer"),
        )
    }

    fn create_surface_configuration(
        surface: &Surface,
        adapter: &Adapter,
        window: &Window,
    ) -> SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let size = window.inner_size();
        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        }
    }

    fn create_pipeline(device: &Device, bind_group_layout: &BindGroupLayout) -> ComputePipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
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

    fn create_textures(device: &Device, window: &Window) -> Texture {
        let size = window.inner_size();
        let texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        device.create_texture(&TextureDescriptor {
            label: Some("Rendered image"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }

    fn create_bind_group(
        device: &Device,
        texture_bind_group_layout: &BindGroupLayout,
        texture_view: &TextureView,
        obj_buffer: &Buffer,
    ) -> BindGroup {
        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: obj_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        texture_bind_group
    }
}
