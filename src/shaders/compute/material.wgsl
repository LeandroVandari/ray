const LAMBERTIAN = 0u;
const METAL = 1u;


struct Material {
    ty: u32,
    albedo: vec3<f32>
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
                scatter_direction =  hit_record.normal;
            }

            (*scattered).ray = Ray(hit_record.point, scatter_direction);
            (*scattered).attenuation = material.albedo;
            return true;
        }

        case METAL: {
            let reflected = reflect(ray.direction, hit_record.normal);

            (*scattered).ray = Ray(hit_record.point, reflected);
            (*scattered).attenuation = material.albedo;
            return true;
        }


        default: {
            return false;
        }
    }

}