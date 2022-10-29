@vertex
fn main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32((in_vertex_index << 1u) & 2u));
    let y = f32(i32(in_vertex_index & 2u));
    let out = 2.0 * vec2(x, y) - vec2(1.0);
    return vec4(out, 0.0, 1.0);
}