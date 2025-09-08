pub const LAMBERTIAN: u32 = 0;
pub const METAL: u32 = 1;
pub const DIELETRIC: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    ty: u32,
    fuzziness: f32,
    refractive_index: f32,
    padding: f32,
    albedo: [f32; 3],
}

const ZERO_MATERIAL: Material = Material {
    ty: 0,
    fuzziness: 0.,
    refractive_index: 0.,
    padding: 0.,
    albedo: [0.; 3],
};

impl Material {
    #[must_use]
    pub const fn lambertian(albedo: [f32; 3]) -> Self {
        Self {
            ty: LAMBERTIAN,
            albedo,
            ..ZERO_MATERIAL
        }
    }

    #[must_use]
    pub const fn metal(albedo: [f32; 3], fuzziness: f32) -> Self {
        Self {
            ty: METAL,
            fuzziness,
            albedo,
            ..ZERO_MATERIAL
        }
    }

    #[must_use]
    pub const fn dieletric(albedo: [f32; 3], refractive_index: f32) -> Self {
        Self {
            ty: DIELETRIC,
            refractive_index,
            albedo,
            ..ZERO_MATERIAL
        }
    }
}
