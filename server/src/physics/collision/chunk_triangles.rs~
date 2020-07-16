use crate::physics::collision::PlanetCollision;

use na::RealField;
use ncollide3d::query::{Contact, ContactKinematic, ContactPreprocessor};
use ncollide3d::shape::Triangle;

pub struct ChunkTriangles<'a> {
    pub planet: &'a PlanetCollision,
    pub samples: Vec<f64>,
    pub coords: shared::planet::Coords,
    pub index: u32,
}

impl<'a> ChunkTriangles<'a> {
    fn vertex(&self, x: u32, y: u32) -> na::Point3<f64> {
        let h = self.samples[(y * self.planet.resolution + x) as usize];
        let quad_resolution = (self.planet.resolution - 1) as f64;
        let unit_coords = na::Point2::new(x as f64 / quad_resolution, y as f64 / quad_resolution);
        let dir = self
            .coords
            .direction(self.planet.terrain.face_resolution(), &unit_coords);
        na::Point3::from(dir.into_inner() * (self.planet.radius + h))
    }
    fn get(&self) -> Triangle<f64> {
        let quad_resolution = self.planet.resolution - 1;
        let quad_index = self.index >> 1;
        let y = quad_index / quad_resolution;
        let x = quad_index % quad_resolution;
        let p0 = self.vertex(x, y);
        let p1 = self.vertex(x + 1, y);
        let p2 = self.vertex(x + 1, y + 1);
        let p3 = self.vertex(x, y + 1);
        let left = (self.index & 1) == 0;
        if left {
            Triangle::new(p0, p1, p2)
        } else {
            Triangle::new(p2, p3, p0)
        }
    }
}

impl Iterator for ChunkTriangles<'_> {
    type Item = Triangle<f64>;
    fn next(&mut self) -> Option<Triangle<f64>> {
        let quad_resolution = self.planet.resolution - 1;
        if self.index == quad_resolution.pow(2) * 2 {
            return None;
        }
        let triangle = self.get();
        self.index += 1;
        Some(triangle)
    }
}

// This part is copypasta tbh
pub struct TriangleContactPreprocessor<'a, N: RealField> {
    pub planet: &'a PlanetCollision,
    pub outer: Option<&'a dyn ContactPreprocessor<N>>,
    pub coords: shared::planet::Coords,
    pub triangle: usize,
}

impl<N: RealField> ContactPreprocessor<N> for TriangleContactPreprocessor<'_, N> {
    fn process_contact(
        &self,
        contact: &mut Contact<N>,
        kinematic: &mut ContactKinematic<N>,
        is_first: bool,
    ) -> bool {
        if is_first {
            kinematic.set_feature1(self.planet.feature_id(
                &self.coords,
                self.triangle,
                kinematic.feature1(),
            ));
        } else {
            kinematic.set_feature2(self.planet.feature_id(
                &self.coords,
                self.triangle,
                kinematic.feature2(),
            ));
        }

        if let Some(x) = self.outer {
            x.process_contact(contact, kinematic, is_first)
        } else {
            true
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkData {
    pub samples: Vec<f64>,
    pub min: f64,
    pub max: f64,
}

impl ChunkData {
    pub fn new(samples: Vec<f64>) -> Self {
        let mut iter = samples.iter().cloned();
        let first = iter.next().unwrap();
        let mut min = first;
        let mut max = first;
        for sample in iter {
            if sample < min {
                min = sample;
            } else if sample > max {
                max = sample;
            }
        }
        Self { samples, min, max }
    }
}
