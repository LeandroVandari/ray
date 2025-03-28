use wgpu::{
    Color, CommandEncoderDescriptor, ComputePassDescriptor, RenderPassColorAttachment,
    RenderPassDescriptor, TextureViewDescriptor,
};
use winit::{application::ApplicationHandler, event::WindowEvent};
mod render_context;
use render_context::RenderContext;

#[derive(Default)]
pub struct App<'window> {
    render_context: Option<RenderContext<'window>>,
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.render_context = Some(pollster::block_on(RenderContext::new(event_loop)).unwrap());
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
                let render_context = self.render_context.as_ref().unwrap();

                let output = render_context.surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&TextureViewDescriptor::default());

                let mut encoder =
                    render_context
                        .device
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("Command Enconder"),
                        });

                {
                    let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                        label: Some("Compute Pass"),
                        timestamp_writes: None,
                    });

                    compute_pass.set_pipeline(&render_context.compute_pipeline);
                    compute_pass.set_bind_group(0, &render_context.bind_group, &[]);
                }

                {
                    let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        ..Default::default()
                    });
                }

                render_context
                    .queue
                    .submit(std::iter::once(encoder.finish()));

                output.present();

                render_context.window.request_redraw();
            }

            _ => (),
        }
    }
}
