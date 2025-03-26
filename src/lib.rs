use std::sync::Arc;

use pollster::FutureExt as _;
use wgpu::{
    Backends, CreateSurfaceError, Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue,
    RequestAdapterOptions, Surface,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    window::{Window, WindowAttributes},
};

#[derive(Default)]
pub struct App<'window> {
    surface_state: Option<SurfaceState<'window>>,
}

struct SurfaceState<'window> {
    window: Arc<Window>,
    surface: Surface<'window>,
    device: Device,
    queue: Queue,
}

impl<'window> SurfaceState<'window> {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Result<Self, CreateSurfaceError> {
        let instance_desc = InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        };
        let instance = Instance::new(&instance_desc);

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_maximized(true)
                        .with_title("Ray tracer"),
                )
                .unwrap(),
        );

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

        Ok(Self {
            surface,
            window,
            device,
            queue,
        })
    }
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.surface_state = Some(SurfaceState::new(event_loop).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                self.surface_state.as_ref().unwrap().window.request_redraw();
            }

            _ => (),
        }
    }
}
