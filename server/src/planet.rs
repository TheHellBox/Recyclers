// Basicly the planet from client side, but without cache manager

use crate::physics::collision::Terrain;

pub struct Planet {
    pub procgen: shared::planet::procgen::PlanetProcGen,
    pub radius: f64,
}

impl Terrain for Planet {
    fn samples(&self, coords: &shared::planet::Coords, resolution: u32) -> Vec<f64> {
        let mut out = Vec::with_capacity(resolution.pow(2) as usize);
        for sample in coords.samples(self.face_resolution(), resolution) {
            out.push(
                self.procgen
                    .get(na::Point3::from(sample.into_inner() * self.radius), u8::MAX)
                    / 12.0,
            );
            //out.push(1000.0)
        }
        out
    }
    fn face_resolution(&self) -> u32 {
        2u32.pow(15)
    }
}
