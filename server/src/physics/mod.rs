use nphysics3d::force_generator::ForceGenerator;
use nphysics3d::object::{BodyHandle, BodySet};

pub mod collision;

const G: f64 = 6.67408e-11;

#[derive(Debug, Clone)]
pub struct PlanetGravity {
    factor: f64,
    position: na::Point3<f64>,
}

impl PlanetGravity {
    pub fn new(mass: f64, position: na::Point3<f64>) -> Self {
        Self {
            factor: G * mass,
            position,
        }
    }
    pub fn set_mass(&mut self, mass: f64) {
        self.factor = G * mass;
    }
    pub fn mass(&mut self) -> f64 {
        self.factor / G
    }
    pub fn set_position(&mut self, position: na::Point3<f64>) {
        self.position = position;
    }
    pub fn position(&mut self) -> &na::Point3<f64> {
        &self.position
    }
}

impl<Handle: BodyHandle> ForceGenerator<f64, Handle> for PlanetGravity {
    fn apply(
        &mut self,
        _params: &nphysics3d::solver::IntegrationParameters<f64>,
        bodies: &mut dyn BodySet<f64, Handle = Handle>,
    ) {
        bodies.foreach_mut(&mut |_, body| {
            for part_id in 0..body.num_parts() {
                let part = match body.part(part_id) {
                    None => break,
                    Some(x) => x,
                };
                let r_2 = na::distance_squared(&self.position, &part.center_of_mass());
                if r_2.abs() < na::convert(1e-3) {
                    continue;
                }
                let magnitude = self.factor / r_2;
                let direction = (self.position - part.center_of_mass()) / r_2.sqrt();
                body.apply_force(
                    part_id,
                    &nphysics3d::math::Force::new(direction * magnitude, na::zero()),
                    nphysics3d::algebra::ForceType::AccelerationChange,
                    false,
                );
            }
        });
    }
}
