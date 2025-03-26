use std::sync::Arc;

use anyhow::{Result, bail};
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages,
};
use winit::window::{Window, WindowAttributes};

pub struct RenderContext<'window> {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: Surface<'window>,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) config: SurfaceConfiguration,
}

impl<'window> RenderContext<'window> {
    pub async fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Result<Self> {
        let instance = Self::create_instance();

        let window = Arc::new(Self::create_window(event_loop)?);

        let surface = instance.create_surface(window.clone())?;

        let adapter = match instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
        {
            Some(adapter) => adapter,
            None => bail!("Couldn't create adapter."),
        };

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .await?;

        let config = Self::create_surface_configuration(&surface, &adapter, &window);
        surface.configure(&device, &config);

        Ok(Self {
            surface,
            window,
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
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        }
    }
}
