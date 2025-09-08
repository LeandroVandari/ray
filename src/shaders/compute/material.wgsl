const LAMBERTIAN = 0u;
const METAL = 1u;
const DIELETRIC = 2u;


struct Material {
    ty: u32,
    fuzziness: f32,
    refractive_index: f32,
    albedo: vec3<f32>,
}

struct ScatteredRay {
    ray: Ray,
    attenuation: vec3<f32>
}

fn scatter(ray: Ray, hit_record: HitRecord, material: Material, scattered: ptr<function, ScatteredRay>, rng_state: ptr<function, u32>) -> bool {
    switch material.ty {
        case LAMBERTIAN: {
            var scatter_direction = hit_record.normal + rngUnitVector(rng_state);

            if near_zero(scatter_direction) {
                scatter_direction = hit_record.normal;
            }

            (*scattered).ray = Ray(hit_record.point, scatter_direction);
            (*scattered).attenuation = material.albedo;
            return true;
        }

        case METAL: {
            var reflected = reflect(ray.direction, hit_record.normal);
            reflected = normalize(reflected) + (material.fuzziness * rngUnitVector(rng_state));

            (*scattered).ray = Ray(hit_record.point, reflected);
            (*scattered).attenuation = material.albedo;
            return dot((*scattered).ray.direction, hit_record.normal) > 0.;
        }

        case DIELETRIC: {
            (*scattered).attenuation = vec3(1.);
            var ri = material.refractive_index;
            if hit_record.front_face {
                ri = 1.0 / material.refractive_index;
            }
            let unit_direction = normalize(ray.direction);

            let cos_theta = min(dot(-unit_direction, hit_record.normal), 1.0);
            let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
            let cannot_refract = ri * sin_theta > 1.0;

            var direction: vec3<f32>;
            if cannot_refract || reflectance(cos_theta, ri) > rngNextFloat(rng_state) {
                direction = reflect(unit_direction, hit_record.normal);
            } else {
                direction = refract(unit_direction, hit_record.normal, ri);
            }

            (*scattered).ray = Ray(hit_record.point, direction);

            return true;
        }


        default: {
            return false;
        }
    }
}

fn reflectance(cosine: f32, refractive_index: f32) -> f32 {
    var r0 = pow((1.0 - refractive_index) / (1.0 + refractive_index), 2.0);
    return fma(1.0 - r0, pow(1. - cosine, 5.), r0);
}