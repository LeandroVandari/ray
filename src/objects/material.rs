pub const LAMBERTIAN: u32 = 0;
pub const METAL: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    ty: u32,
    fuzziness: f32,
    padding: [u32; 2],
    albedo: [f32; 3],
}

impl Material {
    #[must_use]
    pub const fn new(ty: u32, albedo: [f32; 3], fuzziness: Option<f32>) -> Self {
        assert!(ty == LAMBERTIAN || ty == METAL);

        Self {
            ty,
            albedo,
            fuzziness: if let Some(f) = fuzziness { f } else { 0. },
            padding: [0; 2],
        }
    }
}
