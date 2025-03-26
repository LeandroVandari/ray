use std::sync::Arc;

use pollster::FutureExt as _;
use wgpu::{
    Adapter, Backends, CreateSurfaceError, Device, DeviceDescriptor, Instance, InstanceDescriptor,
    Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages,
};
use winit::window::{Window, WindowAttributes};

pub struct RenderContext<'window> {
    pub(crate) window: Arc<Window>,
    surface: Surface<'window>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
}

impl<'window> RenderContext<'window> {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<Self, CreateSurfaceError> {
        let instance = Self::create_instance();

        let window = Arc::new(Self::create_window(event_loop));

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .block_on()
            .unwrap();

        let config = Self::create_surface_configuration(&surface, &adapter, &window);

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

    fn create_window(event_loop: &winit::event_loop::ActiveEventLoop) -> Window {
        event_loop
            .create_window(
                WindowAttributes::default()
                    .with_maximized(true)
                    .with_resizable(false)
                    .with_title("Ray tracer"),
            )
            .unwrap()
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
