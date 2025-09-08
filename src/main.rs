use ray::objects::material;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let spheres = vec![
        ray::objects::Sphere::new(
            [0., -100.5, -1.0],
            100.,
            material::Material::lambertian([0.8, 0.8, 0.]),
        ),
        ray::objects::Sphere::new(
            [0., 0., -1.2],
            0.5,
            material::Material::lambertian([0.1, 0.2, 0.5]),
        ),
        ray::objects::Sphere::new(
            [-1., 0., -1.],
            0.5,
            material::Material::metal([0.8, 0.8, 0.8], 0.3),
        ),
        ray::objects::Sphere::new(
            [1., 0., -1.],
            0.5,
            material::Material::metal([0.8, 0.6, 0.2], 1.0),
        ),
    ];
    let mut app = ray::App::new(spheres);

    event_loop.run_app(&mut app).unwrap();
}
