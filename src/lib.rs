use std::path::Path;

use anyhow::{Result, bail};
use gpu_manager::GpuManager;
use log::info;
use pollster::FutureExt;
pub use render_context::RenderContext;
use renderer::Renderer;
use wgpu::{CommandEncoderDescriptor, Texture};
use winit::{application::ApplicationHandler, event::WindowEvent};

mod compute_context;
pub use compute_context::ComputeContext;
mod render_context;

pub mod objects;
pub mod renderer;

#[cfg(test)]
mod tests;

pub struct App<'window> {
    renderer: Option<Renderer<'window>>,
    spheres: Vec<objects::Sphere>,
}

impl App<'_> {
    #[must_use]
    pub fn new(spheres: Vec<objects::Sphere>) -> Self {
        Self {
            renderer: None,
            spheres,
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.renderer = Some(Renderer::new(event_loop, &self.spheres));
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
                let renderer = self.renderer.take().unwrap();
                drop(renderer);
            }

            WindowEvent::RedrawRequested => {
                let Some(renderer) = self.renderer.as_ref() else {
                    return;
                };

                if renderer.frame() == 0 {
                    return;
                }

                renderer.render();
                renderer.window_manager().window().request_redraw();
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
