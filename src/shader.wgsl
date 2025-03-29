@group(0) @binding(0) var texture: texture_storage_2d<bgra8unorm, write>;
@group(0) @binding(1) var<storage, read> spheres: array<Sphere>;

struct Sphere {
    center: vec3<f32>,
    radius: f32
};

@compute @workgroup_size(8,8,1)
fn main_compute(
    @builtin(global_invocation_id) invocation_id: vec3<u32>, 
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    for (var i = 0u; i < arrayLength(&spheres); i++) {
        if f32(invocation_id.x) >= spheres[i].radius {
            textureStore(texture, location, vec4<f32>(1.0, 0.0, 0.0, 1.0));
        }
    }

}
