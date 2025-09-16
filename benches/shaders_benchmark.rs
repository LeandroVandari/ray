use criterion::{Criterion, criterion_group, criterion_main};
use pollster::FutureExt;
use wgpu::wgt::CommandEncoderDescriptor;

pub fn benchmark(c: &mut Criterion) {
    let mut instantiation = c.benchmark_group("Instantiate managers");
    instantiation.bench_function("Create GpuManager", |b| {
        b.iter(|| gpu_manager::GpuManager::simple().block_on().unwrap());
    });

    let gpu_manager = gpu_manager::GpuManager::simple().block_on().unwrap();

    instantiation.bench_with_input("Compute Context", &gpu_manager, |b, manager| {
        b.iter(|| ray::ComputeContext::new(manager.device(), (1920, 1080), &SPHERES));
    });
    instantiation.finish();

    let mut single_spheres = c.benchmark_group("Single Spheres");
    for num_frames in [1, 5, 10, 60] {
        for output_size in [(128, 128), (256, 256), (512, 512), (1920, 1080)] {
            for (s, sphere) in SPHERES.iter().enumerate() {
                single_spheres.bench_with_input(
                    format!(
                        "Draw sphere #{s} in resolution {output_size:?} in {num_frames} frames."
                    ),
                    &(
                        output_size,
                        *sphere,
                        gpu_manager.device(),
                        gpu_manager.queue(),
                        num_frames,
                    ),
                    |b, (size, sphere, device, queue, frames)| {
                        b.iter_batched(
                            || {
                                ray::ComputeContext::new(
                                    device,
                                    *size,
                                    std::slice::from_ref(sphere),
                                )
                            },
                            |compute_ctx| {
                                for _ in 0..*frames {
                                    let mut encoder = device.create_command_encoder(
                                        &CommandEncoderDescriptor::default(),
                                    );
                                    compute_ctx.draw(&mut encoder, queue);
                                    queue.submit(Some(encoder.finish()));
                                }
                            },
                            criterion::BatchSize::LargeInput,
                        );
                    },
                );
            }
        }
    }

    single_spheres.finish();

    let mut complete_scene = c.benchmark_group("Complete Scene");
    for num_frames in [1, 5, 10, 60] {
        for output_size in [(128, 128), (256, 256), (512, 512), (1920, 1080)] {
            complete_scene.bench_with_input(
                format!("Draw entire scene in resolution {output_size:?} in {num_frames} frames."),
                &(
                    output_size,
                    gpu_manager.device(),
                    gpu_manager.queue(),
                    num_frames,
                ),
                |b, (size, device, queue, frames)| {
                    b.iter_batched(
                        || ray::ComputeContext::new(device, *size, &SPHERES),
                        |compute_ctx| {
                            for _ in 0..*frames {
                                let mut encoder = device
                                    .create_command_encoder(&CommandEncoderDescriptor::default());
                                compute_ctx.draw(&mut encoder, queue);
                                queue.submit(Some(encoder.finish()));
                            }
                        },
                        criterion::BatchSize::LargeInput,
                    );
                },
            );
        }
    }
    complete_scene.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);

const SPHERES: [ray::objects::Sphere; 5] = [
    ray::objects::Sphere::new(
        [0., -100.5, -1.0],
        100.,
        ray::objects::Material::lambertian([0.8, 0.8, 0.]),
    ),
    ray::objects::Sphere::new(
        [0., 0., -1.2],
        0.5,
        ray::objects::Material::lambertian([0.1, 0.2, 0.5]),
    ),
    ray::objects::Sphere::new([-1., 0., -1.], 0.5, ray::objects::Material::dieletric(1.5)),
    ray::objects::Sphere::new(
        [-1., 0., -1.],
        0.4,
        ray::objects::Material::dieletric(1.0 / 1.5),
    ),
    ray::objects::Sphere::new(
        [1., 0., -1.],
        0.5,
        ray::objects::Material::metal([0.8, 0.6, 0.2], 1.0),
    ),
];
