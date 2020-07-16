pub struct Camera {
    pub viewport: nalgebra::Vector4<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            viewport: nalgebra::Vector4::new(0.0, 0.0, 1024.0, 768.0),
        }
    }
    pub fn projection(&self, resolution: (u32, u32), zfar: f32) -> na::Projective3<f32> {
        nproj(
            resolution.0 as f32 / resolution.1 as f32,
            std::f32::consts::PI / 2.0,
            zfar,
        )
    }
}

fn nproj(aspect: f32, vfov: f32, znear: f32) -> na::Projective3<f32> {
    let top = (vfov / 2.0).tan();
    let right = aspect * top;
    let left = -right;
    let bottom = -top;

    let idx = 1.0 / (right - left);
    let idy = 1.0 / (bottom - top);
    let sx = right + left;
    let sy = bottom + top;
    na::Projective3::from_matrix_unchecked(na::Matrix4::new(
        2.0 * idx,
        0.0,
        sx * idx,
        0.0,
        0.0,
        2.0 * idy,
        sy * idy,
        0.0,
        0.0,
        0.0,
        0.0,
        znear,
        0.0,
        0.0,
        -1.0,
        0.0,
    ))
}
