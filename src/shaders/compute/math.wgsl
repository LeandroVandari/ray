const PI = radians(180.0);

const F32_MIN = -3.40282347E+38f;
const F32_MAX = 0x1.fffffep+127f;

fn length_squared(vector: vec3<f32>) -> f32 {
    return pow(vector.x, 2.) + pow(vector.y, 2.) + pow(vector.z, 2.);
}

fn near_zero(vector: vec3<f32>) -> bool {
    let s = 1e-8f;
    let v = abs(vector);
    return any(vec3(v.x<s, v.y < s, v.z < s));

}
