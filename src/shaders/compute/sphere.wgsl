
struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material: Material
};


fn hit_sphere(sphere: Sphere, ray: Ray, interval: Interval, hit_record: ptr<function, HitRecord>) -> bool {
    let oc = sphere.center - ray.origin;
    let a = length_squared(ray.direction);
    let h = dot(ray.direction, oc);
    let c = length_squared(oc) - pow(sphere.radius, 2.);

    let discriminant = h * h - a * c;

    if discriminant < 0 {
        return false;
    }

    let disc_root = sqrt(discriminant);

    var root = (h - disc_root) / a;
    if root <= interval.min || root >= interval.max {
        root = (h + disc_root) / a; 
        if root <= interval.min || root >= interval.max {
            return false;
        }
    }

    (*hit_record).t = root;
    (*hit_record).point = ray_at(ray, root);
    (*hit_record).material = sphere.material;
    let outward_normal = ((*hit_record).point - sphere.center) / sphere.radius;
    set_face_normal(hit_record, ray, outward_normal);

    return true;
}