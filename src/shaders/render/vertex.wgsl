struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

};

@vertex
fn main_vertex(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 4;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 2;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}