@group(0) @binding(0) var texture: texture_storage_2d<bgra8unorm, write>;

struct Sphere {
    @builtin(position) center: vec4<f32>,
    radius: f32
};

@compute @workgroup_size(8,8,1)
fn main(
    @builtin(global_invocation_id) invocation_id: vec3<u32>, 
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    textureStore(texture, location, vec4<f32>(1.0));
}

@fragment
fn output_texture() {
    
}