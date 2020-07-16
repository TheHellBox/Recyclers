use crate::base::components::physics::PhysicsBody;

use std::collections::HashMap;

use ncollide3d::pipeline::broad_phase::DBVTBroadPhase;
use ncollide3d::query::Proximity;
use ncollide3d::shape::ShapeHandle;
use nphysics3d::force_generator::DefaultForceGeneratorSet;
use nphysics3d::joint::DefaultJointConstraintSet;
use nphysics3d::material::BasicMaterial;
use nphysics3d::material::MaterialHandle;
use nphysics3d::object::DefaultBodyHandle;
use nphysics3d::object::{BodyPartHandle, BodyStatus, ColliderDesc, RigidBodyDesc};
use nphysics3d::object::{DefaultBodySet, DefaultColliderSet, RigidBody};
use nphysics3d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

pub struct Physics {
    pub mechanical_world: DefaultMechanicalWorld<f64>,
    pub geometrical_world: DefaultGeometricalWorld<f64>,
    pub joint_constraints: DefaultJointConstraintSet<f64>,
    pub force_generators: DefaultForceGeneratorSet<f64>,
    pub colliders: DefaultColliderSet<f64>,
    pub bodies: DefaultBodySet<f64>,
    pub entities: HashMap<DefaultBodyHandle, hecs::Entity>,
    pub planet_handle: DefaultBodyHandle,
}

impl Physics {
    pub fn new() -> Self {
        // NOTE: At the current state of the development, this hardcode is fine.
        let planet = crate::planet::Planet {
            procgen: shared::planet::procgen::PlanetProcGen::default(),
            radius: 1275620.0,
        };

        let mut mechanical_world = DefaultMechanicalWorld::new(na::zero());
        let geometrical_world = DefaultGeometricalWorld::from_parts(
            DBVTBroadPhase::new(na::convert(0.01)),
            ncollide3d::narrow_phase::NarrowPhase::new(
                Box::new(crate::physics::collision::PlanetDispatcher::new(
                    ncollide3d::narrow_phase::DefaultContactDispatcher::new(),
                )),
                Box::new(ncollide3d::narrow_phase::DefaultProximityDispatcher::new()),
            ),
        );
        let mut bodies = DefaultBodySet::new();
        let mut colliders = DefaultColliderSet::new();
        let joint_constraints = DefaultJointConstraintSet::new();
        let mut force_generators = DefaultForceGeneratorSet::new();

        let planet_handle = bodies.insert(RigidBodyDesc::new().status(BodyStatus::Static).build());
        colliders.insert(
            ColliderDesc::new(ShapeHandle::new(
                crate::physics::collision::PlanetCollision::new(
                    std::sync::Arc::new(planet),
                    8,
                    1275620.0,
                    64 * 1024,
                ),
            ))
            .set_material(MaterialHandle::new(BasicMaterial::new(0.0, 2.0)))
            .build(BodyPartHandle(planet_handle, 0)),
        );

        let gravity_well = crate::physics::PlanetGravity::new(3.0 * 10e22, na::Point3::origin());
        force_generators.insert(Box::new(gravity_well));

        mechanical_world.set_timestep(1.0 / 60.0);
        Self {
            mechanical_world,
            geometrical_world,
            joint_constraints,
            force_generators,
            colliders,
            bodies,
            entities: HashMap::new(),
            planet_handle,
        }
    }
    pub fn add_body(
        &mut self,
        body: RigidBody<f64>,
        entity_builder: &mut hecs::EntityBuilder,
    ) -> DefaultBodyHandle {
        let handle = self.bodies.insert(body);
        let physics_body = PhysicsBody::new(handle);
        entity_builder.add(physics_body);
        handle
    }
    pub fn register_entity(&mut self, handle: DefaultBodyHandle, entity: hecs::Entity) {
        self.entities.insert(handle, entity);
    }
    pub fn run(&mut self, world: &mut hecs::World) {
        use shared::components::Transform;

        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators,
        );
        for (_entity, (physics_handle, mut transform)) in
            &mut world.query::<(&PhysicsBody, &mut Transform)>()
        {
            let body = self
                .bodies
                .get_mut(physics_handle.handle)
                .unwrap()
                .downcast_mut::<RigidBody<f64>>()
                .unwrap();
            let position = body.position();
            transform.isometry = *position;
        }
        for proximity_event in self.geometrical_world.proximity_events() {
            let a_handle = self
                .colliders
                .get(proximity_event.collider1)
                .unwrap()
                .body();
            let b_handle = self
                .colliders
                .get(proximity_event.collider2)
                .unwrap()
                .body();
            println!("{:?} {:?}", a_handle, b_handle);
            if let Some(entity) = self.entities.get(&a_handle) {
                let mut physics_body = world.get_mut::<PhysicsBody>(*entity).unwrap();
                let handles = (proximity_event.collider1, b_handle);
                if proximity_event.new_status == Proximity::Intersecting {
                    physics_body.collides_with.push(handles);
                } else {
                    if let Some(handle) = physics_body
                        .collides_with
                        .iter()
                        .position(|x| *x == handles)
                    {
                        physics_body.collides_with.remove(handle);
                    }
                }
            }
            if let Some(entity) = self.entities.get(&b_handle) {
                let mut physics_body = world.get_mut::<PhysicsBody>(*entity).unwrap();
                let handles = (proximity_event.collider2, a_handle);
                if proximity_event.new_status == Proximity::Intersecting {
                    physics_body.collides_with.push(handles);
                } else {
                    if let Some(handle) = physics_body
                        .collides_with
                        .iter()
                        .position(|x| *x == handles)
                    {
                        physics_body.collides_with.remove(handle);
                    }
                }
            }
        }
        for contact_event in self.geometrical_world.contact_events() {
            use ncollide3d::pipeline::narrow_phase::ContactEvent::*;
            match contact_event {
                Started(a, b) => {
                    let a_is_planet = *a == self.planet_handle;
                    let b_is_planet = *b == self.planet_handle;
                    if a_is_planet || b_is_planet {
                        // 'a' is always a planet
                        let (_a, b) = {
                            if a_is_planet {
                                (a, b)
                            } else {
                                (b, a)
                            }
                        };
                        if let Some(b_entity) = self.entities.get(&b) {
                            let mut physics_handle =
                                world.get_mut::<PhysicsBody>(*b_entity).unwrap();
                            physics_handle.on_surface = true;
                        }
                    }
                }
                Stopped(a, b) => {
                    let a_is_planet = *a == self.planet_handle;
                    let b_is_planet = *b == self.planet_handle;

                    if a_is_planet || b_is_planet {
                        // 'a' is always a planet
                        let (_a, b) = {
                            if a_is_planet {
                                (a, b)
                            } else {
                                (b, a)
                            }
                        };
                        if let Some(b_entity) = self.entities.get(&b) {
                            let mut physics_handle =
                                world.get_mut::<PhysicsBody>(*b_entity).unwrap();
                            physics_handle.on_surface = false;
                        }
                    }
                }
            }
        }
    }
}
