use std::sync::Arc;

use wgpu::{Backends, CreateSurfaceError, Instance, InstanceDescriptor, Surface};
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
    instance: Instance,
    surface: Surface<'window>,
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

        Ok(Self {
            surface: instance.create_surface(window.clone())?,
            instance,
            window,
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
