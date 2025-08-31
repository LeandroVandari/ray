pub const LAMBERTIAN: u32 = 0;
pub const METAL: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    ty: u32,
    albedo: [f32; 3],
}

impl Material {
    pub fn new(ty: u32, albedo: [f32; 3]) -> Self {
        assert!([LAMBERTIAN, METAL].contains(&ty));

        Self { ty, albedo }
    }
}
