@group(0) @binding(0) var texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<storage, read> spheres: array<Sphere>;

fn length_squared(vector: vec3<f32>) -> f32 {
    return pow(vector.x, 2.) + pow(vector.y, 2.) + pow(vector.z, 2.);
}

// TODO: make into a uniform
const SAMPLES_PER_PIXEL = 10u;
const PIXEL_SAMPLES_SCALE = 1.0/f32(SAMPLES_PER_PIXEL);
const FRAME = 0u;

@compute @workgroup_size(8,8,1)
fn main_compute(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    let size = textureDimensions(texture).xy;

    var rng_state = initRng(invocation_id.xy, size, FRAME);

    let camera = create_camera(size);


    let pixCoord = get_pixel_coord(camera, invocation_id.xy);

    var color = vec3(0.);
    for (var i = 0u; i < SAMPLES_PER_PIXEL; i++) {
        let ray = get_ray(camera, invocation_id.x, invocation_id.y, &rng_state);
        color += ray_color(ray);
    }

    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    textureStore(texture, location, vec4<f32>(color*PIXEL_SAMPLES_SCALE, 1.0));
}



fn ray_color(ray: Ray) -> vec3<f32> {
    var hit_record = HitRecord();
    if closest_hit(ray, Interval(0, F32_MAX), &hit_record) {
        return 0.5 * (hit_record.normal + vec3(1.0, 1.0, 1.0));
    }


    let unit_direction = normalize(ray.direction);

    let a = 0.5 * (unit_direction.y + 1.0);
    return (1.0 - a) * vec3<f32>(1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
}

fn closest_hit(ray: Ray, interval: Interval, hit_record: ptr<function, HitRecord>) -> bool {
    var temp_rec = HitRecord();
    var hit_anything = false;
    var closest_so_far = interval.max;

    for (var i = 0u; i < arrayLength(&spheres); i++) {
        if hit_sphere(spheres[i], ray, Interval(interval.min, closest_so_far), &temp_rec) {
            hit_anything = true;
            closest_so_far = temp_rec.t;
            *hit_record = temp_rec;
        }
    }

    return hit_anything;
}