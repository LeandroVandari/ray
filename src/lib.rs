use log::info;
use render_context::RenderContext;
use wgpu::{
    CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, Origin3d, RenderPassDescriptor,
    TexelCopyTextureInfoBase, TextureFormat, TextureViewDescriptor,
};
use winit::{application::ApplicationHandler, event::WindowEvent};

mod render_manager;
use render_manager::RenderManager;

mod compute_context;
use compute_context::ComputeContext;
mod render_context;

pub mod objects;

pub struct App<'window> {
    render_manager: Option<RenderManager<'window>>,
    compute_context: Option<ComputeContext>,
    render_context: Option<RenderContext>,
    spheres: Vec<objects::Sphere>,
}

impl App<'_> {
    pub fn new(spheres: Vec<objects::Sphere>) -> Self {
        Self {
            render_manager: None,
            compute_context: None,
            render_context: None,
            spheres,
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.render_manager = Some(pollster::block_on(RenderManager::new(event_loop)).unwrap());

        let render_manager = self.render_manager.as_ref().unwrap();
        let window_size = render_manager.window.inner_size();

        self.compute_context = Some(ComputeContext::new(
            &render_manager.device,
            (window_size.width, window_size.height),
            TextureFormat::Rgba8Unorm,
            &self.spheres,
        ));

        self.render_context = Some(RenderContext::new(
            &render_manager.device,
            self.compute_context.as_ref().unwrap(),
            render_manager,
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
                let render_manager = self.render_manager.take().unwrap();
                drop(render_manager);
            }

            WindowEvent::RedrawRequested => {
                let render_manager = self.render_manager.as_ref().unwrap();
                let compute_context = self.compute_context.as_ref().unwrap();
                let render_context = self.render_context.as_ref().unwrap();

                let output = render_manager.surface.get_current_texture().unwrap();

                let mut encoder =
                    render_manager
                        .device
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

                render_manager
                    .queue
                    .submit(std::iter::once(encoder.finish()));

                output.present();

                render_manager.window.request_redraw();
            }

            _ => (),
        }
    }
}
