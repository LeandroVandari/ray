use std::sync::Arc;

use anyhow::{Result, bail};
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::window::{Window, WindowAttributes};

pub struct RenderManager<'window> {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: Surface<'window>,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) config: SurfaceConfiguration,
}

impl RenderManager<'_> {
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
                    required_features: Features::empty(),
                    ..Default::default()
                },
                None,
            )
            .await?;

        let config = Self::create_surface_configuration(&surface, &adapter, &window)?;
        surface.configure(&device, &config);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
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
    ) -> Result<SurfaceConfiguration> {
        let surface_caps = surface.get_capabilities(adapter);
        dbg!(&surface_caps);
        let usage = if surface_caps.usages.contains(TextureUsages::COPY_DST) {
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST
        } else {
            log::warn!("Surface can't be copy destination. Using compatibility mode.");
            TextureUsages::RENDER_ATTACHMENT
        };

        fn get_surface_format(available_formats: &[TextureFormat]) -> Result<TextureFormat> {
            let priority_formats = [
                wgpu::TextureFormat::Rgba8Unorm,
                wgpu::TextureFormat::Bgra8Unorm,
            ];
            for format in priority_formats {
                if available_formats.contains(&format) {
                    return Ok(format);
                }
            }
            bail!("Couldn't get supported surface format, exiting.");
        }

        let surface_format = get_surface_format(&surface_caps.formats)?;

        let size = window.inner_size();
        Ok(SurfaceConfiguration {
            usage,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        })
    }
}
