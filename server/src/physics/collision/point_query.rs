use crate::physics::collision::chunk_triangles::*;
use crate::physics::collision::PlanetCollision;
use ncollide3d::query::PointProjection;
use ncollide3d::query::PointQuery;
use ncollide3d::shape::FeatureId;

// idk how it works. I just copied it from Ralith code
impl PointQuery<f64> for PlanetCollision {
    fn project_point(
        &self,
        m: &na::Isometry3<f64>,
        pt: &na::Point3<f64>,
        solid: bool,
    ) -> PointProjection<f64> {
        if solid && na::distance_squared(pt, &(m * na::Point3::origin())) < self.radius {
            return PointProjection {
                is_inside: true,
                point: *pt,
            };
        };
        self.project_point_with_feature(m, pt).0
    }

    fn project_point_with_feature(
        &self,
        m: &na::Isometry3<f64>,
        pt: &na::Point3<f64>,
    ) -> (PointProjection<f64>, FeatureId) {
        let local = m.inverse_transform_point(pt);
        let coords = shared::planet::Coords::from_vector(
            self.terrain.face_resolution(),
            &na::convert(local.coords),
        );
        let distance2 = |x: &na::Point3<f64>| na::distance_squared(x, &local);
        let cache = &mut *self.cache.lock().unwrap();
        let data = if let Some(x) = cache.get(&coords) {
            x
        } else {
            cache.put(
                coords,
                ChunkData::new(self.terrain.samples(&coords, self.resolution)),
            );
            cache.get(&coords).unwrap()
        };
        let (idx, (nearest, feature)) = ChunkTriangles {
            planet: self,
            samples: data.samples.clone(),
            coords: coords,
            index: 0,
        }
        .map(|tri| tri.project_point_with_feature(m, &local))
        .enumerate()
        .min_by(|(_, (x, _)), (_, (y, _))| {
            distance2(&x.point)
                .partial_cmp(&distance2(&y.point))
                .unwrap()
        })
        .unwrap();
        (
            PointProjection {
                point: m * nearest.point,
                ..nearest
            },
            self.feature_id(&coords, idx, feature),
        )
    }
}
