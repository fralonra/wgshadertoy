#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    pub cursor: [f32; 2],
    pub mouse_down: u32,
    _padding0: u32,
    pub mouse_press: [f32; 2],
    pub mouse_release: [f32; 2],
    pub resolution: [f32; 2],
    pub time: f32,
    _padding1: u32,
}

impl Uniform {
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}
