// Heavily inspired by fantastic work of Ralith, the planetmap
// https://github.com/Ralith/planetmap

pub mod cache;
pub mod chunk;
pub mod procgen;

use core::ops::Neg;
use std::cmp::Ordering;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Face {
    PX,
    NX,
    PY,
    NY,
    PZ,
    NZ,
}

impl Neg for Face {
    type Output = Self;
    fn neg(self) -> Self {
        use self::Face::*;
        match self {
            PX => NX,
            PY => NY,
            PZ => NZ,
            NX => PX,
            NY => PY,
            NZ => PZ,
        }
    }
}

impl Face {
    pub fn basis(self) -> na::Rotation3<f64> {
        use Face::*;
        let (x, y, z) = match self {
            PX => (na::Vector3::z(), -na::Vector3::y(), na::Vector3::x()),
            NX => (-na::Vector3::z(), -na::Vector3::y(), -na::Vector3::x()),
            PY => (na::Vector3::x(), -na::Vector3::z(), na::Vector3::y()),
            NY => (na::Vector3::x(), na::Vector3::z(), -na::Vector3::y()),
            PZ => (na::Vector3::x(), na::Vector3::y(), na::Vector3::z()),
            NZ => (-na::Vector3::x(), na::Vector3::y(), -na::Vector3::z()),
        };
        na::Rotation3::from_matrix_unchecked(na::Matrix3::from_columns(&[x, y, z]))
    }
    pub fn from_vector(x: &na::Vector3<f64>) -> Self {
        let (&value, &axis) = x
            .iter()
            .zip(&[Face::PX, Face::PY, Face::PZ])
            .max_by(|(l, _), (r, _)| l.abs().partial_cmp(&r.abs()).unwrap_or(Ordering::Less))
            .unwrap();
        if value.is_sign_negative() {
            -axis
        } else {
            axis
        }
    }
    pub fn coords(x: &na::Vector3<f64>) -> (Face, na::Point2<f64>) {
        let face = Self::from_vector(x);
        let wrt_face = face.basis().inverse_transform_vector(x);
        (
            face,
            na::Point2::from(wrt_face.xy() * (na::convert::<_, f64>(0.5) / wrt_face.z))
                + na::convert::<_, na::Vector2<f64>>(na::Vector2::new(0.5, 0.5)),
        )
    }

    pub fn direction(self, coords: &na::Point2<f64>) -> na::Unit<na::Vector3<f64>> {
        let dir_z = na::Unit::new_normalize(na::Vector3::new(coords.x, coords.y, 1.0));
        self.basis() * dir_z
    }
    pub fn iter() -> impl Iterator<Item = Face> {
        const VALUES: &[Face] = &[Face::PX, Face::NX, Face::PY, Face::NY, Face::PZ, Face::NZ];
        VALUES.iter().cloned()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Coords {
    pub coords: (u32, u32),
    pub face: Face,
}

impl Coords {
    pub fn direction(
        &self,
        resolution: u32,
        coords: &na::Point2<f64>,
    ) -> na::Unit<na::Vector3<f64>> {
        let edge_length = edge_length(resolution);
        let origin = na::Vector2::new(self.coords.0 as f64, self.coords.1 as f64) * edge_length
            - na::Vector2::repeat(1.0);
        let position = origin + coords.coords * edge_length;
        self.face.direction(&position.into())
    }
    // Samples without x and y information
    pub fn samples(
        &self,
        face_resolution: u32,
        chunk_resolution: u32,
    ) -> Vec<na::Unit<na::Vector3<f64>>> {
        let step = 1.0 / (chunk_resolution - 1) as f64;
        let mut result = Vec::with_capacity(chunk_resolution.pow(2) as usize);
        for y in 0..chunk_resolution {
            for x in 0..chunk_resolution {
                result.push(self.direction(
                    face_resolution,
                    &(na::Point2::new(x as f64, y as f64) * step),
                ));
            }
        }
        result
    }
    // Samples with x and y information
    pub fn samples_xy(
        &self,
        face_resolution: u32,
        chunk_resolution: u32,
    ) -> Vec<(na::Unit<na::Vector3<f64>>, (u32, u32))> {
        let step = 1.0 / (chunk_resolution - 1) as f64;
        let mut result = Vec::with_capacity(chunk_resolution.pow(2) as usize);
        for y in 0..chunk_resolution {
            for x in 0..chunk_resolution {
                result.push((
                    self.direction(
                        face_resolution,
                        &(na::Point2::new(x as f64, y as f64) * step),
                    ),
                    (x, y),
                ));
            }
        }
        result
    }

    pub fn from_vector(resolution: u32, vector: &na::Vector3<f64>) -> Self {
        let (face, unit_coords) = Face::coords(vector);
        let (x, y) = discretize(resolution as usize, unit_coords);
        Self {
            coords: (x as u32, y as u32),
            face,
        }
    }

    // Copy-pasta from planetmap. I actually wrote my own version of this function, but it didn't work, so I raged and just copied the original
    pub fn neighborhood(
        resolution: u32,
        direction: na::Vector3<f64>,
        theta: f64,
    ) -> impl Iterator<Item = Self> {
        Face::iter()
            .filter(move |f| {
                (f.basis() * na::Vector3::z())
                    .dot(&direction)
                    .is_sign_positive()
            })
            .map(move |face| {
                let local = face.basis().inverse_transform_vector(&direction);
                let local = local.xy() / local.z;
                // atan(x / 1) = angle of `local` around Y axis through cube origin ("midpoint x")
                let theta_m_x = local.x.atan();
                // tan(θ_mx - θ) * 1 = coordinate of the intersection of the X lower bound with the cube
                let x_lower = (theta_m_x - theta).tan();
                // tan(θ_mx + θ) * 1 = coordinate of the intersection of the X upper bound with the cube
                let x_upper = (theta_m_x + theta).tan();
                // once more, perpendicular!
                let theta_m_y = local.y.atan();
                let y_lower = (theta_m_y - theta).tan();
                let y_upper = (theta_m_y + theta).tan();
                (face, (x_lower, y_lower), (x_upper, y_upper))
            })
            .filter(|(_, lower, upper)| {
                lower.0 <= 1.0 && lower.1 <= 1.0 && upper.0 >= -1.0 && upper.1 >= -1.0
            })
            .flat_map(move |(face, lower, upper)| {
                let (x_lower, y_lower) = discretize(
                    resolution as usize,
                    na::Point2::new(remap(lower.0), remap(lower.1)),
                );
                let (x_upper, y_upper) = discretize(
                    resolution as usize,
                    na::Point2::new(remap(upper.0), remap(upper.1)),
                );
                (y_lower..=y_upper).flat_map(move |y| {
                    (x_lower..=x_upper).map(move |x| Self {
                        coords: (x as u32, y as u32),
                        face,
                    })
                })
            })
    }
}

pub fn edge_length(resolution: u32) -> f64 {
    2.0 / resolution as f64
}

fn discretize(resolution: usize, texcoords: na::Point2<f64>) -> (usize, usize) {
    let texcoords = texcoords * resolution as f64;
    let max = resolution - 1;
    (
        na::clamp(texcoords.x as usize, 0, max),
        na::clamp(texcoords.y as usize, 0, max),
    )
}

fn remap(x: f64) -> f64 {
    (na::clamp(x, -1.0, 1.0) + 1.0) / 2.0
}
