use std::f32::consts::PI;

use ray::objects::material;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let r: f32 = (PI / 4.).cos();

    let spheres = vec![
        ray::objects::Sphere::new(
            [-r, 0., -1.],
            r,
            material::Material::lambertian([0., 0., 1.]),
        ),
        ray::objects::Sphere::new(
            [r, 0., -1.],
            r,
            material::Material::lambertian([1., 0., 0.]),
        ),
    ];
    let mut app = ray::App::new(spheres);

    event_loop.run_app(&mut app).unwrap();
}
