use log::info;
use wgpu::{
    CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, Origin3d, TexelCopyTextureInfoBase,
    TextureFormat,
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
    spheres: Vec<objects::Sphere>,
}

impl App<'_> {
    pub fn new(spheres: Vec<objects::Sphere>) -> Self {
        Self {
            render_manager: None,
            compute_context: None,
            spheres,
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.render_manager = Some(pollster::block_on(RenderManager::new(event_loop)).unwrap());

        let render_context = self.render_manager.as_ref().unwrap();
        let window_size = render_context.window.inner_size();

        self.compute_context = Some(ComputeContext::new(
            &render_context.device,
            (window_size.width, window_size.height),
            TextureFormat::Rgba8Unorm,
            &self.spheres,
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
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfoBase {
                        texture: &compute_context.output_texture,
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
                        width: compute_context.output_texture.width(),
                        height: compute_context.output_texture.height(),
                        depth_or_array_layers: 1,
                    },
                );

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
