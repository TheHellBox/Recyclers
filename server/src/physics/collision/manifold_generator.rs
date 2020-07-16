use crate::physics::collision::chunk_triangles::*;
use crate::physics::collision::PlanetCollision;

use std::collections::hash_map;
use std::collections::HashMap;

use ncollide3d::bounding_volume::BoundingVolume;
use ncollide3d::narrow_phase::ContactAlgorithm;
use ncollide3d::narrow_phase::ContactDispatcher;
use ncollide3d::narrow_phase::ContactManifoldGenerator;
use ncollide3d::query::{ContactManifold, ContactPrediction, ContactPreprocessor};
use ncollide3d::shape::Shape;

pub struct PlanetManifoldGenerator {
    state: HashMap<(shared::planet::Coords, usize), ContactAlgorithm<f64>>,
    flip: bool,
}

impl PlanetManifoldGenerator {
    pub fn new(flip: bool) -> Self {
        Self {
            flip,
            state: HashMap::new(),
        }
    }
    pub fn run(
        &mut self,
        dispatcher: &dyn ContactDispatcher<f64>,
        m_a: &na::Isometry3<f64>,
        planet: &PlanetCollision,
        cpreproc1: Option<&dyn ContactPreprocessor<f64>>,
        m_b: &na::Isometry3<f64>,
        other: &dyn Shape<f64>,
        cpreproc2: Option<&dyn ContactPreprocessor<f64>>,
        prediction: &ContactPrediction<f64>,
        manifold: &mut ContactManifold<f64>,
    ) {
        // Find bounds
        let bounds = other.bounding_sphere(m_b).loosened(prediction.linear());
        // Find distance from m_a to m_b
        let dir = m_a.inverse_transform_point(bounds.center()).coords;
        let distance = dir.norm();
        let cache = &mut *planet.cache.lock().unwrap();

        for coords in shared::planet::Coords::neighborhood(
            planet.terrain.face_resolution(),
            na::convert(dir),
            bounds.radius().atan2(distance) as f64,
        ) {
            let data = {
                if let Some(samples) = cache.get(&coords) {
                    samples
                } else {
                    cache.put(
                        coords,
                        ChunkData::new(planet.terrain.samples(&coords, planet.resolution)),
                    );
                    cache.get(&coords).unwrap()
                }
            };
            if planet.radius + data.max + bounds.radius() < distance {
                continue;
            }
            let triangles = ChunkTriangles {
                planet: planet,
                samples: data.samples.clone(),
                index: 0,
                coords,
            };

            for (i, triangle) in triangles
                .enumerate()
                .filter(|(_, x)| x.bounding_sphere(m_a).intersects(&bounds))
            {
                let tri = match self.state.entry((coords, i)) {
                    hash_map::Entry::Occupied(e) => e.into_mut(),
                    hash_map::Entry::Vacant(e) => {
                        if let Some(algo) = if self.flip {
                            dispatcher.get_contact_algorithm(other, &triangle)
                        } else {
                            dispatcher.get_contact_algorithm(&triangle, other)
                        } {
                            e.insert(algo)
                        } else {
                            return;
                        }
                    }
                };
                let proc1 = TriangleContactPreprocessor {
                    planet,
                    outer: cpreproc1,
                    coords,
                    triangle: i,
                };

                if !self.flip {
                    tri.generate_contacts(
                        dispatcher,
                        m_a,
                        &triangle,
                        Some(&proc1),
                        m_b,
                        other,
                        cpreproc2,
                        prediction,
                        manifold,
                    );
                } else {
                    tri.generate_contacts(
                        dispatcher,
                        m_b,
                        other,
                        cpreproc2,
                        m_a,
                        &triangle,
                        Some(&proc1),
                        prediction,
                        manifold,
                    );
                }
            }
        }
    }
}

impl ContactManifoldGenerator<f64> for PlanetManifoldGenerator {
    fn generate_contacts(
        &mut self,
        d: &dyn ContactDispatcher<f64>,
        m_a: &na::Isometry3<f64>,
        a: &dyn Shape<f64>,
        cpreproc1: Option<&dyn ContactPreprocessor<f64>>,
        m_b: &na::Isometry3<f64>,
        b: &dyn Shape<f64>,
        cpreproc2: Option<&dyn ContactPreprocessor<f64>>,
        prediction: &ContactPrediction<f64>,
        manifold: &mut ContactManifold<f64>,
    ) -> bool {
        if !self.flip {
            if let Some(p) = a.as_shape::<PlanetCollision>() {
                self.run(
                    d, m_a, p, cpreproc1, m_b, b, cpreproc2, prediction, manifold,
                );
                return true;
            }
        } else {
            if let Some(p) = b.as_shape::<PlanetCollision>() {
                self.run(
                    d, m_b, p, cpreproc2, m_a, a, cpreproc1, prediction, manifold,
                );
                return true;
            }
        }
        false
    }
}
