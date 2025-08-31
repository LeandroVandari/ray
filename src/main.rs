use ray::objects::material;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let spheres = vec![
        ray::objects::Sphere::new(
            [0., 0., -1.0],
            0.5,
            material::Material::new(material::METAL, [1., 1., 1.]),
        ),
        ray::objects::Sphere::new(
            [0.0, -100.5, -1.0],
            100.,
            material::Material::new(material::LAMBERTIAN, [1., 1., 1.]),
        ),
    ];
    let mut app = ray::App::new(spheres);

    event_loop.run_app(&mut app).unwrap();
}
