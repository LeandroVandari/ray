@group(0) @binding(0) var texture: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(0) var<storage, read> spheres: array<Sphere>;
@group(1) @binding(1) var<uniform> frame: u32;

fn length_squared(vector: vec3<f32>) -> f32 {
    return pow(vector.x, 2.) + pow(vector.y, 2.) + pow(vector.z, 2.);
}

const MAGENTA = vec3(0.74, 0.02, 0.84);

// TODO: make into a uniform
const SAMPLES_PER_PIXEL = 10u;
const PIXEL_SAMPLES_SCALE = 1.0/f32(SAMPLES_PER_PIXEL);
const MAX_RAY_BOUNCES = 50u;

@compute @workgroup_size(8,8,1)
fn main_compute(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    let size = textureDimensions(texture).xy;

    var rng_state = initRng(invocation_id.xy, size, frame);

    let camera = create_camera(size);


    let pixCoord = get_pixel_coord(camera, invocation_id.xy);

    var color = vec3(0.);
    for (var i = 0u; i < SAMPLES_PER_PIXEL; i++) {
        let ray = get_ray(camera, invocation_id.x, invocation_id.y, &rng_state);
        color += ray_color(ray, &rng_state);
    }

    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    // sqrt: convert to gamma space
    textureStore(texture, location, vec4<f32>(sqrt(color*PIXEL_SAMPLES_SCALE), 1.0));
}



fn ray_color(ray: Ray, state: ptr<function, u32>) -> vec3<f32> {

    var hit_record = HitRecord();
    
    var new_ray = ray;

    var bounces = 0u;
    var color = vec3(0.);
    for (var bounce = 0u; bounce < MAX_RAY_BOUNCES; bounce++) {
        if closest_hit(new_ray, Interval(0.001, F32_MAX), &hit_record) {
            let direction = hit_record.normal + rngUnitVector(state);
            new_ray.origin = hit_record.point;
            new_ray.direction = direction;
            bounces++;
            continue;
        }
        else {
            let unit_direction = normalize(new_ray.direction);
            let a = 0.5 * (unit_direction.y + 1.0);
            color = (1.0 - a) * vec3<f32>(1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
            break;
        }


    }
    return color * pow(0.5, f32(bounces));
    
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