@group(0) @binding(0) var texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<storage, read> spheres: array<Sphere>;

fn length_squared(vector: vec3<f32>) -> f32 {
    return pow(vector.x, 2.) + pow(vector.y, 2.) + pow(vector.z, 2.);
}


@compute @workgroup_size(8,8,1)
fn main_compute(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    let size = textureDimensions(texture).xy;

    let camera = create_camera(size);


    let pixCoord = get_pixel_coord(camera, invocation_id.xy);


    let ray_dir = pixCoord - CAMERA_CENTER;
    let ray = Ray(CAMERA_CENTER, ray_dir);

    let color = ray_color(ray);


    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    textureStore(texture, location, vec4<f32>(color, 1.0));
}



fn ray_color(ray: Ray) -> vec3<f32> {
    for (var i = 0u ; i < arrayLength(&spheres); i++) {
        let t = hit_sphere(spheres[i], ray);

        if t > 0.0 {
            let normal = normalize(ray_at(ray, t) - spheres[i].center);
            return 0.5 * vec3(normal + 1.);
        }
    }


    let unit_direction = normalize(ray.direction);

    let a = 0.5 * (unit_direction.y + 1.0);
    return (1.0 - a) * vec3<f32>(1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
}