use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Transform {
    pub isometry: na::Isometry3<f64>,
    pub scale: na::Vector3<f64>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            isometry: na::Isometry3::from_parts(
                na::Translation3::new(0.0, 0.0, 0.0),
                na::UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
            ),
            scale: na::Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn transform_matrix(&self) -> na::Matrix4<f64> {
        self.isometry.to_homogeneous() * scale_matrix(self.scale)
    }
    pub fn calculate_view(&self) -> na::Isometry3<f64> {
        self.isometry.inverse()
    }
}

pub fn scale_matrix(scale: nalgebra::Vector3<f64>) -> nalgebra::Matrix4<f64> {
    nalgebra::Matrix4::new(
        scale.x, 0.0, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, 0.0, scale.z, 0.0, 0.0, 0.0, 0.0, 1.0,
    )
}
