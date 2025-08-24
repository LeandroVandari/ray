@group(0) @binding(0) var texture: texture_2d<f32>;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

};

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureLoad(texture, vec2(u32(in.clip_position.x), u32(in.clip_position.y)), 0);
}