// Based on https://github.com/Ralith/planetmap/blob/master/src/ncollide.rs

/*
big FIXME:

So... This part of the code is mainly copy-pasta from planetmap.
In fact, it's worse than the original in many places.
It needs A LOT of improvements, first of all, code cleanup.
Then, after the code looks clean, there's lots of places where it can be optimized.
Also, lots of things are not implemented at all, like raycasts(Those are imposible as far as I know)
*/

pub mod chunk_triangles;
pub mod manifold_generator;
pub mod point_query;
pub mod tests;

pub use chunk_triangles::*;
pub use manifold_generator::*;
pub use point_query::*;

use na::RealField;
use ncollide3d::bounding_volume::{BoundingSphere, HasBoundingVolume, AABB};
use ncollide3d::narrow_phase::ContactAlgorithm;
use ncollide3d::narrow_phase::ContactDispatcher;
use ncollide3d::query::PointQuery;
use ncollide3d::shape::{FeatureId, Shape};
use std::sync::Mutex;

pub trait Terrain: Sync + Send {
    fn samples(&self, coords: &shared::planet::Coords, resolution: u32) -> Vec<f64>;
    fn face_resolution(&self) -> u32;
}

pub struct PlanetCollision {
    resolution: u32,
    cache: Mutex<lru::LruCache<shared::planet::Coords, ChunkData>>,
    terrain: std::sync::Arc<dyn Terrain>,
    radius: f64,
}

impl Clone for PlanetCollision {
    fn clone(&self) -> Self {
        Self {
            terrain: self.terrain.clone(),
            cache: Mutex::new(lru::LruCache::new(self.cache.lock().unwrap().cap())),
            ..*self
        }
    }
}

impl PlanetCollision {
    pub fn new(
        terrain: std::sync::Arc<dyn Terrain>,
        resolution: u32,
        radius: f64,
        cache_size: usize,
    ) -> Self {
        Self {
            terrain,
            resolution,
            radius,
            cache: Mutex::new(lru::LruCache::new(cache_size)),
        }
    }
    fn feature_id(
        &self,
        _coords: &shared::planet::Coords,
        _triangle: usize,
        _tri_feature: FeatureId,
    ) -> FeatureId {
        FeatureId::Unknown
    }
}

impl<N: RealField> HasBoundingVolume<N, BoundingSphere<N>> for PlanetCollision {
    fn bounding_volume(&self, m: &na::Isometry3<N>) -> BoundingSphere<N> {
        BoundingSphere::new(m * na::Point3::origin(), na::convert(self.radius + 32000.0))
    }
}

impl<N: RealField> HasBoundingVolume<N, AABB<N>> for PlanetCollision {
    fn bounding_volume(&self, m: &na::Isometry3<N>) -> AABB<N> {
        let radius = na::convert(self.radius + 32000.0);
        AABB::from_half_extents(
            m * na::Point3::origin(),
            na::Vector3::new(radius, radius, radius),
        )
    }
}

impl Shape<f64> for PlanetCollision {
    fn aabb(&self, m: &na::Isometry3<f64>) -> AABB<f64> {
        self.bounding_volume(m)
    }
    fn bounding_sphere(&self, m: &na::Isometry3<f64>) -> BoundingSphere<f64> {
        self.bounding_volume(m)
    }
    // TODO: implement this
    fn tangent_cone_contains_dir(
        &self,
        _fid: FeatureId,
        _m: &na::Isometry3<f64>,
        _deformations: Option<&[f64]>,
        _dir: &na::Unit<na::Vector3<f64>>,
    ) -> bool {
        false
    }
    fn as_point_query(&self) -> Option<&dyn PointQuery<f64>> {
        Some(self)
    }
}

pub struct PlanetDispatcher<T> {
    inner: T,
}

impl<T> PlanetDispatcher<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: ContactDispatcher<f64>> ContactDispatcher<f64> for PlanetDispatcher<T> {
    fn get_contact_algorithm(
        &self,
        a: &dyn Shape<f64>,
        b: &dyn Shape<f64>,
    ) -> Option<ContactAlgorithm<f64>> {
        if a.is_shape::<PlanetCollision>() {
            return Some(Box::new(PlanetManifoldGenerator::new(false)));
        }
        if b.is_shape::<PlanetCollision>() {
            return Some(Box::new(PlanetManifoldGenerator::new(true)));
        }
        self.inner.get_contact_algorithm(a, b)
    }
}
