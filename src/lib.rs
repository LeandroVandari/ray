use log::info;
use render_context::RenderContext;
use wgpu::{
    CommandEncoderDescriptor, ComputePassDescriptor, RenderPassDescriptor, TextureViewDescriptor,
};
use winit::{application::ApplicationHandler, event::WindowEvent};

mod gpu_manager;
pub use gpu_manager::{GpuManager, WindowManager};

mod compute_context;
use compute_context::ComputeContext;
mod render_context;

pub mod objects;

#[cfg(test)]
mod tests;

pub struct App<'window> {
    gpu_manager: Option<GpuManager<WindowManager<'window>>>,
    compute_context: Option<ComputeContext>,
    render_context: Option<RenderContext>,
    spheres: Vec<objects::Sphere>,
    frame: u32,
}

impl App<'_> {
    pub fn new(spheres: Vec<objects::Sphere>) -> Self {
        Self {
            gpu_manager: None,
            compute_context: None,
            render_context: None,
            spheres,
            frame: 0,
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.gpu_manager = Some(pollster::block_on(GpuManager::with_window(event_loop)).unwrap());

        let gpu_manager = self.gpu_manager.as_ref().unwrap();
        let window_size = gpu_manager.window().inner_size();

        self.compute_context = Some(ComputeContext::new(
            gpu_manager.device(),
            (window_size.width, window_size.height),
            &self.spheres,
        ));

        self.render_context = Some(RenderContext::new(
            gpu_manager.device(),
            self.compute_context.as_ref().unwrap(),
            gpu_manager.config().format,
        ))
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Exiting window");
                event_loop.exit();
                let compute_context = self.compute_context.take().unwrap();
                drop(compute_context);
                let render_context = self.render_context.take().unwrap();
                drop(render_context);
                let gpu_manager = self.gpu_manager.take().unwrap();
                drop(gpu_manager);
            }

            WindowEvent::RedrawRequested => {
                self.frame += 1;
                let (Some(gpu_manager), Some(compute_context), Some(render_context)) = (
                    self.gpu_manager.as_ref(),
                    self.compute_context.as_ref(),
                    self.render_context.as_ref(),
                ) else {
                    return;
                };
                gpu_manager.queue().write_buffer(
                    &compute_context.frame_uniform,
                    0,
                    &self.frame.to_be_bytes(),
                );

                let output = gpu_manager.surface().get_current_texture().unwrap();

                let mut encoder =
                    gpu_manager
                        .device()
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("Command Enconder"),
                        });
                {
                    let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                        label: Some("Compute Pass"),
                        timestamp_writes: None,
                    });

                    compute_pass.set_pipeline(&compute_context.compute_pipeline);
                    compute_pass.set_bind_group(0, &compute_context.bind_group, &[]);
                    compute_pass.dispatch_workgroups(
                        compute_context.output_texture.width() / 8 + 1,
                        compute_context.output_texture.height() / 8 + 1,
                        1,
                    );
                }

                {
                    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("RenderPass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &output
                                .texture
                                .create_view(&TextureViewDescriptor::default()),
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
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    render_pass.set_bind_group(0, &render_context.bind_group, &[]);
                    render_pass.set_pipeline(&render_context.pipeline);
                    render_pass.draw(0..3, 0..3);
                }

                gpu_manager
                    .queue()
                    .submit(std::iter::once(encoder.finish()));

                output.present();

                gpu_manager.window().request_redraw();
            }

            _ => (),
        }
    }
}
