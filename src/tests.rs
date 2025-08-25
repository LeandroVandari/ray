use pollster::FutureExt;
use wgpu::TextureFormat;

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
