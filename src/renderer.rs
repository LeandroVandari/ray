use std::sync::{Arc, atomic::AtomicBool};

use gpu_manager::{GpuManager, WindowManager};
use wgpu::{CommandEncoderDescriptor, wgt::TextureViewDescriptor};

use crate::{ComputeContext, RenderContext, objects::Sphere};

pub struct Renderer<'window> {
    gpu_manager: Arc<GpuManager<()>>,
    draw_handle: Arc<AtomicBool>,
    window_manager: WindowManager<'window>,
    render_context: RenderContext,
}

impl<'window> Renderer<'window> {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop, spheres: &[Sphere]) -> Self {
        log::info!("Creating Renderer...");
        log::trace!("Creating GpuManager...");
        let (gpu_manager, window_manager) = pollster::block_on(GpuManager::with_window(event_loop))
            .unwrap()
            .split();

        let window_size = window_manager.window().inner_size();

        log::trace!("Creating ComputeContext...");
        let compute_context = ComputeContext::new(
            gpu_manager.device(),
            (window_size.width, window_size.height),
            spheres,
        );

        log::trace!("Creating RenderContext...");
        let render_context = RenderContext::new(
            gpu_manager.device(),
            &compute_context,
            window_manager.config().format,
        );

        let gpu_manager = Arc::new(gpu_manager);

        let draw_handle = Arc::new(AtomicBool::new(true));

        let (draw_handlet, gpu_managert) = (draw_handle.clone(), gpu_manager.clone());
        log::trace!("Creating compute thread...");
        let _compute_thread = std::thread::spawn(move || {
            loop {
                while !draw_handlet.swap(false, std::sync::atomic::Ordering::Acquire) {}
                run_compute_shader(&gpu_managert, &compute_context);
            }
        });

        log::info!("Done creating Renderer...");
        Self {
            draw_handle,
            window_manager,
            gpu_manager,
            render_context,
        }
    }

    pub fn render(&self) {
        log::info!("Running render shader...");
        let output = self.window_manager.surface().get_current_texture().unwrap();
        let mut encoder =
            self.gpu_manager
                .device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Command Enconder"),
                });
        self.render_context.draw_to_texture(
            &mut encoder,
            &output
                .texture
                .create_view(&TextureViewDescriptor::default()),
        );

        self.gpu_manager.queue().submit(Some(encoder.finish()));
        log::info!("Finished render shader...");
        output.present();
        self.draw_handle
            .store(true, std::sync::atomic::Ordering::Release);
    }

    pub fn gpu_manager(&self) -> &GpuManager<()> {
        &self.gpu_manager
    }

    pub fn window_manager(&self) -> &WindowManager<'window> {
        &self.window_manager
    }

    pub fn frame(&self) -> u32 {
        self.render_context.frame()
    }
}

fn run_compute_shader(gpu_manager: &GpuManager, compute_context: &ComputeContext) {
    log::info!("Running compute shader...");
    let mut encoder = gpu_manager
        .device()
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Compute Command Enconder"),
        });

    compute_context.draw(&mut encoder, gpu_manager.queue());

    gpu_manager
        .queue()
        .submit(std::iter::once(encoder.finish()));
    log::info!("Submit compute shader to queue...");
}
