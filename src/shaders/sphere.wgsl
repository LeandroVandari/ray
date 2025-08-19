
struct Sphere {
    center: vec3<f32>,
    radius: f32
};


fn hit_sphere(sphere: Sphere, ray: Ray) -> f32 {
    let oc = sphere.center - ray.origin;
    let a = length_squared(ray.direction);
    let h = dot(ray.direction, oc);
    let c = length_squared(oc) - pow(sphere.radius, 2.);

    let discriminant = h * h - a * c;

    if discriminant < 0 {
        return -1.0;
    } else {
        return (h - sqrt(discriminant)) / a;
    }
}