
struct HitRecord {
    point: vec3<f32>,
    normal: vec3<f32>,
    t: f32,
    front_face: bool,
    material: Material
};

fn set_face_normal(record: ptr<function, HitRecord>, ray: Ray, outward_normal: vec3<f32>) {
    (*record).front_face = dot(ray.direction, outward_normal) < 0;

    if (*record).front_face {
        (*record).normal = outward_normal;
    } else {
        (*record).normal = - outward_normal;
    }
}