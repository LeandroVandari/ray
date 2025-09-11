const CAMERA_CENTER: vec3<f32> = vec3<f32>(0.0);

struct Camera {
    pix0_coord: vec3<f32>,
    pixel_delta_u: vec3<f32>,
    pixel_delta_v: vec3<f32>
}

fn get_pixel_coord(camera: Camera, invocation_id: vec2<u32>) -> vec3<f32> {
    return camera.pix0_coord + (f32(invocation_id.x) * camera.pixel_delta_u) + (f32(invocation_id.y) * camera.pixel_delta_v);
}

fn create_camera(size: vec2<u32>) -> Camera {
    let focal_length = 1.0;
    let fov = 90.0;
    let theta = radians(fov);
    let h = tan(theta / 2.);
    let viewport_height = 2 * h * focal_length;
    let viewport_width = viewport_height * (f32(size.x) / f32(size.y));

    let viewport_u = vec3<f32>(viewport_width, 0.0, 0.0);
    let viewport_v = vec3<f32>(0.0, -viewport_height, 0.0);

    let pixel_delta_u = viewport_u / f32(size.x);
    let pixel_delta_v = viewport_v / f32(size.y);

    let viewport_upper_left = CAMERA_CENTER - vec3<f32>(0., 0., focal_length) - viewport_u / 2. - viewport_v / 2.;

    let pix0_coord = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    return Camera(pix0_coord, pixel_delta_u, pixel_delta_v);
}

fn get_ray(camera: Camera, i: u32, j: u32, rng_state: ptr<function, u32>) -> Ray {
    let offset = sample_square(rng_state);

    let pixel_sample = camera.pix0_coord + ((f32(i) + offset.x) * camera.pixel_delta_u) + ((f32(j) + offset.y) * camera.pixel_delta_v);

    let ray_origin = CAMERA_CENTER;
    let ray_direction = pixel_sample - ray_origin;

    return Ray(ray_origin, ray_direction);
}

fn sample_square(rng_state: ptr<function, u32>) -> vec2<f32> {
    return vec2(rngNextFloat(rng_state) - 0.5, rngNextFloat(rng_state) - 0.5);
} 