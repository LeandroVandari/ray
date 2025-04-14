@group(0) @binding(0) var texture: texture_storage_2d<bgra8unorm, write>;
@group(0) @binding(1) var<storage, read> spheres: array<Sphere>;

struct Sphere {
    center: vec3<f32>,
    radius: f32
};

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32>{
    return ray.origin + ray.direction*t;
}
fn length_squared(vector: vec3<f32>) -> f32 {
    return pow(vector.x,2.) + pow(vector.y,2.) + pow(vector.z,2.);
}


struct Camera {
    pix0_coord: vec3<f32>,
    pixel_delta_u: vec3<f32>,
    pixel_delta_v: vec3<f32>
}

fn get_pixel_coord(camera: Camera, invocation_id: vec2<u32>) -> vec3<f32> {
    return camera.pix0_coord + (f32(invocation_id.x) * camera.pixel_delta_u) + (f32(invocation_id.y) * camera.pixel_delta_v);
}

const CAMERA_CENTER: vec3<f32> = vec3<f32>(0.0);

@compute @workgroup_size(8,8,1)
fn main_compute(
    @builtin(global_invocation_id) invocation_id: vec3<u32>, 
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    let size = textureDimensions(texture).xy;

    let camera = create_camera(size);


    let pixCoord = get_pixel_coord(camera, invocation_id.xy);


    let ray_dir = pixCoord - CAMERA_CENTER;
    let ray = Ray(CAMERA_CENTER,ray_dir);

    let color = ray_color(ray);


    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    textureStore(texture, location, vec4<f32>(color, 1.0));

}


fn ray_color(ray: Ray) -> vec3<f32> {
    for (var i=0u ;i<arrayLength(&spheres);i++) {
        let t = hit_sphere(spheres[i], ray);

        if (t > 0.0) {
            let normal = normalize(ray_at(ray, t) - spheres[i].center);
            return 0.5*vec3(normal + 1.);
        }
    }
    

    let unit_direction = normalize(ray.direction);

    let a = 0.5*(unit_direction.y+1.0);
    return (1.0-a)*vec3<f32>(1.0) + a*vec3<f32>(0.5, 0.7, 1.0);

}

fn hit_sphere(sphere: Sphere, ray: Ray) -> f32 {
    let oc = sphere.center - ray.origin;
    let a = length_squared(ray.direction);
    let h = dot(ray.direction, oc);
    let c = length_squared(oc) - pow(sphere.radius,2.);

    let discriminant = h*h - a*c;

    if (discriminant < 0) {
        return -1.0;
    }
    else {
        return (h-sqrt(discriminant))/a;
    }
}




fn create_camera(size: vec2<u32>) -> Camera {
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height*(f32(size.x)/f32(size.y));

    let viewport_u = vec3<f32>(viewport_width, 0.0, 0.0);
    let viewport_v = vec3<f32>(0.0, -viewport_height, 0.0);

    let pixel_delta_u = viewport_u / f32(size.x);
    let pixel_delta_v = viewport_v / f32(size.y);

    let viewport_upper_left = CAMERA_CENTER - vec3<f32>(0.,0.,focal_length) - viewport_u/2. - viewport_v/2.;

    let pix0_coord = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    return Camera(pix0_coord, pixel_delta_u, pixel_delta_v);
}
