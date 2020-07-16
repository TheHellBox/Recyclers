pub mod pickable;

use crate::base::components::PhysicsBody;
use crate::base::game_manager::GameManager;
use nphysics3d::object::{Body, BodyPartHandle, ColliderDesc, RigidBody, RigidBodyDesc};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Clone)]
pub enum JSONShape {
    Cuboid(f64, f64, f64),
    ConvexHull(String),
    Capsule { half_height: f64, radius: f64 },
    Cylinder { half_height: f64, radius: f64 },
    Ball(f64),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ColliderDescJSON {
    shape: JSONShape,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PropData {
    model: String,
    #[serde(default = "default_mass")]
    mass: f64,
    collider_desc: ColliderDescJSON,
}

impl GameManager {
    pub fn spawn_prop(&mut self, owner: &PhysicsBody, prop_id: usize) {
        use ncollide3d::shape::*;
        use shared::components::{Drawable, Transform};

        let prop_data = self.props[&prop_id].clone();

        let player_position = self
            .physics
            .bodies
            .get(owner.handle)
            .unwrap()
            .downcast_ref::<RigidBody<f64>>()
            .unwrap()
            .position()
            .clone();

        let mut prop = hecs::EntityBuilder::new();
        prop.add(Transform {
            isometry: na::Isometry3::from_parts(
                na::Translation3::new(0.0, 0.0, 0.0),
                na::UnitQuaternion::identity(),
            ),
            ..Default::default()
        });
        prop.add(self.new_id());
        prop.add(Drawable {
            model: prop_data.model,
            shader: "SIMPLE".to_string(),
        });
        prop.add(pickable::PickAble { owner: None });
        let view_direction = player_position.rotation * &na::Vector3::new(0.0, 0.0, -1.0);
        let prop_body = self.physics.add_body(
            RigidBodyDesc::new()
                .mass(prop_data.mass)
                .translation(player_position.translation.vector + view_direction * 2.0)
                .build(),
            &mut prop,
        );
        let shape = {
            match prop_data.collider_desc.shape {
                // FIXME: Don't create new shape handle for each object
                JSONShape::Cuboid(x, y, z) => {
                    ShapeHandle::new(Cuboid::new(na::Vector3::new(x, y, z)))
                }
                JSONShape::Ball(radius) => ShapeHandle::new(Ball::new(radius)),
                // FIXME: That's a cylinder, not and capsule. ncollide just does not implement shape trait for cylinder
                JSONShape::Cylinder {
                    half_height,
                    radius,
                } => ShapeHandle::new(Capsule::new(half_height, radius)),
                JSONShape::Capsule {
                    half_height,
                    radius,
                } => ShapeHandle::new(Capsule::new(half_height, radius)),
                JSONShape::ConvexHull(path) => ShapeHandle::new(
                    crate::base::gltf_loader::load_convex(std::path::Path::new(&path)),
                ),
            }
        };

        let collider_desc = ColliderDesc::new(shape.clone()).density(1.0);
        let sensor_desc = ColliderDesc::new(shape).sensor(true).margin(1.0);

        self.physics
            .colliders
            .insert(collider_desc.build(BodyPartHandle(prop_body, 0)));
        self.physics
            .colliders
            .insert(sensor_desc.build(BodyPartHandle(prop_body, 0)));

        let body = self
            .physics
            .bodies
            .get_mut(prop_body)
            .unwrap()
            .downcast_mut::<RigidBody<f64>>()
            .unwrap();
        body.apply_force(
            0,
            &nphysics3d::math::Force::new(view_direction * 1000.0, na::zero()),
            nphysics3d::algebra::ForceType::Force,
            true,
        );

        let entity = self.world.spawn(prop.build());
        self.physics.register_entity(prop_body, entity);
        self.spawn(entity);
    }
    pub fn load_props(&mut self) {
        use std::path::Path;
        let plate = load_prop_data(Path::new("./assets/props/plate_1x1.json")).unwrap();
        let wheel = load_prop_data(Path::new("./assets/props/wheel.json")).unwrap();
        self.props.insert(0, plate);
        self.props.insert(1, wheel);
    }
}

fn load_prop_data(path: &std::path::Path) -> Result<PropData, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let prop_data: PropData = serde_json::from_reader(reader)?;
    Ok(prop_data)
}

fn default_mass() -> f64 {
    1.0
}
