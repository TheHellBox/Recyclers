use crate::physics::collision::Terrain;

#[derive(Debug, Copy, Clone)]
pub struct FlatTerrain {
    face_resolution: u32,
}

impl Terrain for FlatTerrain {
    fn samples(&self, coords: &shared::planet::Coords, resolution: u32) -> Vec<f64> {
        let mut out = Vec::with_capacity(resolution.pow(2) as usize);
        for _sample in coords.samples(self.face_resolution(), resolution) {
            out.push(0.0)
        }
        out
    }
    fn face_resolution(&self) -> u32 {
        2u32.pow(12)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::collision::*;
    use ncollide3d::{
        narrow_phase::{DefaultContactDispatcher, DefaultProximityDispatcher, NarrowPhase},
        pipeline::{
            object::{CollisionGroups, GeometricQueryType},
            world::CollisionWorld,
        },
        shape::{Ball, ShapeHandle},
    };
    use std::sync::Arc;

    #[test]
    fn coordinate_center_regression() {
        let mut world = CollisionWorld::new(0.01);
        world.set_narrow_phase(NarrowPhase::new(
            Box::new(PlanetDispatcher::new(DefaultContactDispatcher::new())),
            Box::new(DefaultProximityDispatcher::new()),
        ));

        const PLANET_RADIUS: f64 = 6371e3;
        const BALL_RADIUS: f64 = 50.0;

        world.add(
            na::Isometry3::identity(),
            ShapeHandle::new(PlanetCollision::new(
                Arc::new(FlatTerrain {
                    face_resolution: 2u32.pow(12),
                }),
                17,
                PLANET_RADIUS,
                32,
            )),
            CollisionGroups::new(),
            GeometricQueryType::Contacts(0.01, 0.01),
            0,
        );
        let coords = na::Vector3::<f64>::new(-5_195_083.148, 3_582_099.812, -877_091.267)
            .normalize()
            * PLANET_RADIUS as f64;
        let (_ball, _) = world.add(
            na::convert(na::Translation3::from(coords)),
            ShapeHandle::new(Ball::new(BALL_RADIUS)),
            CollisionGroups::new(),
            GeometricQueryType::Contacts(0.01, 0.01),
            0,
        );

        world.update();
        assert!(world.contact_pairs(true).count() > 0);
    }
}
