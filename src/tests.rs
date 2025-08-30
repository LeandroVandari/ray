use std::path::Path;

use pollster::FutureExt;
use wgpu::{CommandEncoderDescriptor, TextureFormat};

use crate::{
    compute_context::ComputeContext, gpu_manager::GpuManager, objects::Sphere,
    render_context::RenderContext,
};

#[test]
fn create_gpu_manager() {
    assert!(pollster::block_on(GpuManager::simple()).is_ok());
}

#[test]
fn create_compute_context() {
    let gpu_manager = GpuManager::simple().block_on().unwrap();

    let compute_ctx = ComputeContext::new(
        gpu_manager.device(),
        (100, 100),
        &[Sphere::new([0., 0., 0.], 3.0)],
    );

    dbg!(compute_ctx);
}

#[test]
fn create_render_context() {
    let gpu_manager = GpuManager::simple().block_on().unwrap();
    let compute_ctx = ComputeContext::new(
        gpu_manager.device(),
        (100, 100),
        &[Sphere::new([0., 0., 0.], 3.0)],
    );

    let render_ctx = RenderContext::new(
        gpu_manager.device(),
        &compute_ctx,
        TextureFormat::Bgra8Unorm,
    );

    dbg!(render_ctx);
}

#[test]
fn draw_scene() {
    let gpu_manager = GpuManager::simple().block_on().unwrap();

    let compute_ctx = ComputeContext::new(
        gpu_manager.device(),
        (100, 100),
        &[
            Sphere::new([0., 0., -1.0], 0.5),
            Sphere::new([0.0, -100.5, -1.0], 100.),
        ],
    );

    let mut encoder = gpu_manager
        .device()
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Test Encoder"),
        });

    compute_ctx.draw(&mut encoder);
}

#[test]
fn render_to_file() {
    let gpu_manager = GpuManager::simple().block_on().unwrap();

    let compute_ctx = ComputeContext::new(
        gpu_manager.device(),
        // Must be a multiple of 256 (1920 works ???)
        (1920, 1080),
        &[
            Sphere::new([0., 0., -1.0], 0.5),
            Sphere::new([0.0, -100.5, -1.0], 100.),
        ],
    );

    let mut encoder = gpu_manager
        .device()
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Test Encoder"),
        });

    compute_ctx.draw(&mut encoder);
    gpu_manager.queue().submit(Some(encoder.finish()));

    assert!(
        super::write_to_file(
            &gpu_manager,
            &compute_ctx.output_texture,
            Some(Path::new("test_output.png"))
        )
        .is_ok()
    );
}
