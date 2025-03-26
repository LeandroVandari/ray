use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
};
mod render_context;
use render_context::RenderContext;

#[derive(Default)]
pub struct App<'window> {
    surface_state: Option<RenderContext<'window>>,
}



impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.surface_state = Some(RenderContext::new(event_loop).unwrap());
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
