use std::path::Path;

use gpu_manager::GpuManager;
use pollster::FutureExt;
use wgpu::{CommandEncoderDescriptor, TextureFormat};

use crate::{
    compute_context::ComputeContext,
    objects::{Sphere, material},
    render_context::RenderContext,
};

const SPHERES: [Sphere; 4] = [
    Sphere::new(
        [0., -100.5, -1.0],
        100.,
        material::Material::new(material::LAMBERTIAN, [0.8, 0.8, 0.], None),
    ),
    Sphere::new(
        [0., 0., -1.2],
        0.5,
        material::Material::new(material::LAMBERTIAN, [0.1, 0.2, 0.5], None),
    ),
    Sphere::new(
        [-1., 0., -1.],
        0.5,
        material::Material::new(material::METAL, [0.8, 0.8, 0.8], Some(0.3)),
    ),
    Sphere::new(
        [1., 0., -1.],
        0.5,
        material::Material::new(material::METAL, [0.8, 0.6, 0.2], Some(1.0)),
    ),
];

#[test]
fn create_compute_context() {
    let gpu_manager = GpuManager::simple().block_on().unwrap();

    let compute_ctx = ComputeContext::new(gpu_manager.device(), (100, 100), &SPHERES);

    dbg!(compute_ctx);
}

#[test]
fn create_render_context() {
    let gpu_manager = GpuManager::simple().block_on().unwrap();
    let compute_ctx = ComputeContext::new(gpu_manager.device(), (100, 100), &SPHERES);

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

    let compute_ctx = ComputeContext::new(gpu_manager.device(), (100, 100), &SPHERES);

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
        // Width must be a multiple of 128
        (128, 128),
        &SPHERES,
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
            &compute_ctx.previous_texture,
            Some(Path::new("one_frame_test.png"))
        )
        .is_ok()
    );
}

#[test]
fn render_multiple_frames() {
    let gpu_manager = GpuManager::simple().block_on().unwrap();

    let compute_ctx = ComputeContext::new(
        gpu_manager.device(),
        // Width must be a multiple of 128
        (128, 128),
        &SPHERES,
    );

    for i in 0u32..10 {
        let mut encoder = gpu_manager
            .device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Test Encoder"),
            });
        gpu_manager.queue().write_buffer(
            &compute_ctx.frame_uniform,
            0,
            &[i.to_be_bytes(), [0; 4], [0; 4], [0; 4]].concat(),
        );
        compute_ctx.draw(&mut encoder);
        gpu_manager.queue().submit(Some(encoder.finish()));
    }

    assert!(
        super::write_to_file(
            &gpu_manager,
            &compute_ctx.output_texture,
            Some(Path::new("multiple_frames_test.png"))
        )
        .is_ok()
    );
}
