use super::Material;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    center: [f32; 3],
    radius: f32,
    material: Material,
    other_padding: [u32; 1],
}

impl Sphere {
    pub fn new(center: [f32; 3], radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
            other_padding: [0; 1],
        }
    }
}
