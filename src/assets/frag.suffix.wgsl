@fragment
fn main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let base_color = vec4(0.0, 0.0, 0.0, 1.0);
    let color = main_image(base_color, (frag_coord.xy - u.resolution) * vec2(1.0, -1.0));
    return vec4(color.rgb, 1.0);
}