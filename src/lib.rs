use std::path::Path;

use anyhow::{Result, bail};
use gpu_manager::{GpuManager, WindowManager};
use log::info;
use pollster::FutureExt;
use render_context::RenderContext;
use wgpu::{CommandEncoderDescriptor, Texture, TextureViewDescriptor};
use winit::{application::ApplicationHandler, event::WindowEvent};

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
    #[must_use]
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
        ));
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
                    // Uniform buffers must be aligned to 16 bytes
                    &[self.frame.to_be_bytes(), [0; 4], [0; 4], [0; 4]].concat(),
                );

                let output = gpu_manager.surface().get_current_texture().unwrap();

                let mut encoder =
                    gpu_manager
                        .device()
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("Command Enconder"),
                        });

                compute_context.draw(&mut encoder);
                render_context.draw_to_texture(
                    &mut encoder,
                    &output
                        .texture
                        .create_view(&TextureViewDescriptor::default()),
                );

                gpu_manager
                    .queue()
                    .submit(std::iter::once(encoder.finish()));
                output.present();
                self.frame += 1;
                gpu_manager.window().request_redraw();
            }

            _ => (),
        }
    }
}

pub fn write_to_file<SurfaceManager>(
    gpu_manager: &GpuManager<SurfaceManager>,
    texture: &Texture,
    path: Option<&Path>,
) -> Result<()> {
    const U32_SIZE: u32 = std::mem::size_of::<u32>() as u32;

    let output_buffer_size = (U32_SIZE * texture.width() * texture.height()) as wgpu::BufferAddress;

    let output_buffer_desc = wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    };

    let output_buffer = gpu_manager.device().create_buffer(&output_buffer_desc);

    let mut encoder = gpu_manager
        .device()
        .create_command_encoder(&CommandEncoderDescriptor::default());

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfoBase {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfoBase {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(U32_SIZE * texture.width()),
                rows_per_image: Some(texture.height()),
            },
        },
        texture.size(),
    );

    gpu_manager.queue().submit(Some(encoder.finish()));

    {
        let buffer_slice = output_buffer.slice(..);

        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());

        gpu_manager.device().poll(wgpu::PollType::Wait)?;

        rx.receive().block_on();

        let data = buffer_slice.get_mapped_range();

        use image::{ImageBuffer, Rgba};
        let Some(buffer) =
            ImageBuffer::<Rgba<u8>, _>::from_raw(texture.width(), texture.height(), data)
        else {
            bail!("Couldn't save image to file.")
        };

        buffer.save(path.unwrap_or(Path::new("output.png")))?;
    }

    output_buffer.unmap();

    Ok(())
}
