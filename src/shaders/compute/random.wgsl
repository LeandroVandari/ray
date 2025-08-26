fn rngNextVec3InUnitSphere(state: ptr<function, u32>) -> vec3<f32> {
    let r = pow(rngNextFloat(state), 0.33333f);
    let cosTheta = 1f - 2f * rngNextFloat(state);
    let sinTheta = sqrt(1f - cosTheta * cosTheta);
    let phi = 2f * PI * rngNextFloat(state);

    let x = r * sinTheta * cos(phi);
    let y = r * sinTheta * sin(phi);
    let z = cosTheta;

    return vec3(x, y, z);
}

fn rngNextFloat(state: ptr<function, u32>) -> f32 {
    let x = rngNextInt(state);
    return f32(x) / f32(0xffffffffu);
}

fn rngUnitVector(state: ptr<function, u32>) -> vec3<f32> {
    let v = vec3(rngNextFloat(state), rngNextFloat(state), rngNextFloat(state));

    return normalize(v);
}

fn initRng(pixel: vec2<u32>, resolution: vec2<u32>, frame: u32) -> u32 {
    // Adapted from https://github.com/boksajak/referencePT
    let seed = dot(pixel, vec2<u32>(1u, resolution.x)) ^ jenkinsHash(frame);
    return jenkinsHash(seed);
}

fn rngNextInt(state: ptr<function, u32>) -> u32 {
    // PCG random number generator
    // Based on https://www.shadertoy.com/view/XlGcRh
    let newState = *state * 747796405u + 2891336453u;
    *state = newState;
    let word = ((newState >> ((newState >> 28u) + 4u)) ^ newState) * 277803737u;
    return (word >> 22u) ^ word;
}

fn jenkinsHash(input: u32) -> u32 {
    var x = input;
    x += x << 10u;
    x ^= x >> 6u;
    x += x << 3u;
    x ^= x >> 11u;
    x += x << 15u;
    return x;
}

fn rngNextVec3OnHemisphere(state: ptr<function, u32>, normal: vec3<f32>) -> vec3<f32> {
    let on_unit_sphere = rngNextVec3InUnitSphere(state);

    if dot(on_unit_sphere, normal) > 0.0 {
        return on_unit_sphere;
    } else {
        return -on_unit_sphere;
    }
}