# WgShadertoy

A [WGSL](https://www.w3.org/TR/WGSL/) playground inspired by [Shadertoy](https://www.shadertoy.com/).

<img src="https://i.ibb.co/GV2rwTj/Wg-Shadertoy-001.png" alt="preview">

## Wgs format

The application use a binary format `wgs` to save and load shaders and textures.

It helps to share your shaders amoung people.

You can find examples in [wgshader-examples](https://github.com/fralonra/wgshader-examples).

## Uniforms

The runtime currently provided six parameters you can use in your shader as a uniform variable:

- `cursor`: _vec2<f32>_
  - The mouse position in pixels.
- `mouse_down`: _u32_
  - Whether the left button of the mouse is down.
  - `0`: left button is up.
  - `1`: left button is down.
- `mouse_press`: _vec2<f32>_
  - The mouse position in pixels when the left button is pressed.
- `mouse_release`: _vec2<f32>_
  - The mouse position in pixels when the left button is released.
- `resolution`: _vec2<f32>_
  - The resolution of the canvas in pixels (width \* height).
- `time`: _f32_
  - The elapsed time since the shader first ran, in seconds.

You can use the above uniform like the following:

```wgsl
fn main_image(frag_color: vec4<f32>, frag_coord: vec2<f32>) -> vec4<f32> {
    let uv = frag_coord / u.resolution;
    let color = 0.5 + 0.5 * cos(u.time + uv.xyx + vec3(0.0, 2.0, 4.0));
    return vec4(color, 1.0);
}
```

## Limits

- The amount of the texture you can upload is [the max bind group count of your device](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html#structfield.max_bind_groups) - 1.
- Invalid shader would panic!

## Credits

- [wgpu](https://github.com/gfx-rs/wgpu) for rendering.
- [egui](https://github.com/emilk/egui) for UI.
- [binrw](https://github.com/jam1garner/binrw) for binary data read/write.
- [shadertoy](https://github.com/adamnemecek/shadertoy) for the wonderful vertex shader.
