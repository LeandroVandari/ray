use std::{f32::consts::PI, path::Path};

use gpu_manager::GpuManager;
use pollster::FutureExt;
use wgpu::{CommandEncoderDescriptor, TextureFormat};

use crate::{
    compute_context::ComputeContext,
    objects::{Sphere, material},
    render_context::RenderContext,
};

const SPHERES: [Sphere; 5] = [
    Sphere::new(
        [0., -100.5, -1.0],
        100.,
        material::Material::lambertian([0.8, 0.8, 0.]),
    ),
    Sphere::new(
        [0., 0., -1.2],
        0.5,
        material::Material::lambertian([0.1, 0.2, 0.5]),
    ),
    Sphere::new([-1., 0., -1.], 0.5, material::Material::dieletric(1.5)),
    Sphere::new(
        [-1., 0., -1.],
        0.4,
        material::Material::dieletric(1.0 / 1.5),
    ),
    Sphere::new(
        [1., 0., -1.],
        0.5,
        material::Material::metal([0.8, 0.6, 0.2], 1.0),
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

    compute_ctx.draw(&mut encoder, gpu_manager.queue());
}

#[test]
fn render_materials_to_file() {
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

    compute_ctx.draw(&mut encoder, gpu_manager.queue());
    gpu_manager.queue().submit(Some(encoder.finish()));

    assert!(
        super::write_to_file(
            &gpu_manager,
            &compute_ctx.previous_texture,
            Some(Path::new("one_frame_materials_test.png"))
        )
        .is_ok()
    );
}

#[test]
fn render_fov_to_file() {
    let gpu_manager = GpuManager::simple().block_on().unwrap();

    let r: f32 = (PI / 4.).cos();

    let spheres = vec![
        Sphere::new(
            [-r, 0., -1.],
            r,
            material::Material::lambertian([0., 0., 1.]),
        ),
        Sphere::new(
            [r, 0., -1.],
            r,
            material::Material::lambertian([1., 0., 0.]),
        ),
    ];

    let compute_ctx = ComputeContext::new(
        gpu_manager.device(),
        // Width must be a multiple of 128
        (128, 128),
        &spheres,
    );

    let mut encoder = gpu_manager
        .device()
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Test Encoder"),
        });

    compute_ctx.draw(&mut encoder, gpu_manager.queue());
    gpu_manager.queue().submit(Some(encoder.finish()));

    assert!(
        super::write_to_file(
            &gpu_manager,
            &compute_ctx.previous_texture,
            Some(Path::new("one_frame_fov_test.png"))
        )
        .is_ok()
    );
}

#[test]
fn render_multiple_frames_materials() {
    let gpu_manager = GpuManager::simple().block_on().unwrap();

    let compute_ctx = ComputeContext::new(
        gpu_manager.device(),
        // Width must be a multiple of 128
        (128, 128),
        &SPHERES,
    );

    for i in 0u32..60 {
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
        compute_ctx.draw(&mut encoder, gpu_manager.queue());
        gpu_manager.queue().submit(Some(encoder.finish()));
    }

    assert!(
        super::write_to_file(
            &gpu_manager,
            &compute_ctx.output_texture,
            Some(Path::new("multiple_frames_materials_test.png"))
        )
        .is_ok()
    );
}

#[test]
fn render_multiple_frames_fov() {
    let gpu_manager = GpuManager::simple().block_on().unwrap();
    let r: f32 = (PI / 4.).cos();

    let spheres = vec![
        Sphere::new(
            [-r, 0., -1.],
            r,
            material::Material::lambertian([0., 0., 1.]),
        ),
        Sphere::new(
            [r, 0., -1.],
            r,
            material::Material::lambertian([1., 0., 0.]),
        ),
    ];

    let compute_ctx = ComputeContext::new(
        gpu_manager.device(),
        // Width must be a multiple of 128
        (128, 128),
        &spheres,
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
        compute_ctx.draw(&mut encoder, gpu_manager.queue());
        gpu_manager.queue().submit(Some(encoder.finish()));
    }

    assert!(
        super::write_to_file(
            &gpu_manager,
            &compute_ctx.output_texture,
            Some(Path::new("multiple_frames_fov_test.png"))
        )
        .is_ok()
    );
}
