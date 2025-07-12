
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub color: [f32; 4],
    pub position: [f32; 3],
    pub _padding: f32,
}

