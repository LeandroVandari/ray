use wgpu::{
    CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, Origin3d, TexelCopyTextureInfoBase,
};
use winit::{application::ApplicationHandler, event::WindowEvent};
mod render_context;
use render_context::RenderContext;

#[derive(Default)]
pub struct App<'window> {
    render_context: Option<RenderContext<'window>>,
}

impl ApplicationHandler for App<'_> {
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
                let render_context = self.render_context.take().unwrap();
                drop(render_context);
            }

            WindowEvent::RedrawRequested => {
                let render_context = self.render_context.as_ref().unwrap();

                let output = render_context.surface.get_current_texture().unwrap();

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
                    compute_pass.dispatch_workgroups(
                        render_context.texture.width() / 8 + 1,
                        render_context.texture.height() / 8 + 1,
                        1,
                    );
                }

                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfoBase {
                        texture: &render_context.texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    TexelCopyTextureInfoBase {
                        texture: &output.texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    Extent3d {
                        width: render_context.texture.width(),
                        height: render_context.texture.height(),
                        depth_or_array_layers: 1,
                    },
                );

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
