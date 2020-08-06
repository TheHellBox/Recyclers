use crate::base::components::PhysicsBody;
use crate::base::systems::physics::Physics;
use nphysics3d::object::{Body, BodyStatus, RigidBody};
use shared::components::*;

use ncollide3d::shape::{Ball, Cuboid, ShapeHandle};
use nphysics3d::material::{BasicMaterial, MaterialHandle};
use nphysics3d::object::{
    BodyPartHandle, ColliderDesc, DefaultBodyHandle, DefaultColliderHandle, RigidBodyDesc,
};

pub struct Player {
    pub name: String,
    pub state: Option<shared::commands::ClientCommand>,
    pub picked_object: Option<hecs::Entity>,
    pub ground_sensor: DefaultColliderHandle,
}

impl Player {
    pub fn new(name: String, ground_sensor: DefaultColliderHandle) -> Self {
        Self {
            name,
            ground_sensor,
            state: None,
            picked_object: None,
        }
    }
    pub fn walk(
        &self,
        physics: &mut Physics,
        handle: &PhysicsBody,
        planet_handle: DefaultBodyHandle,
    ) {
        let state = {
            if let Some(s) = self.state {
                s
            } else {
                return;
            }
        };
        let on_surface = handle.collides_with.len() > 0;
        let body = physics
            .bodies
            .get_mut(handle.handle)
            .unwrap()
            .downcast_mut::<RigidBody<f64>>()
            .unwrap();

        // Convert from i8 to float. Normalize to avoid cheating
        let mut movement_direction = na::Vector3::new(
            state.movement_direction.x as f64,
            0.0,
            -state.movement_direction.y as f64,
        )
        .try_normalize(0.5)
        .unwrap_or(na::Vector3::repeat(0.0));

        if state.run {
            movement_direction *= 2.0;
        }
        if state.sit {
            movement_direction *= 0.2;
        }
        if state.jump && state.sit {
            movement_direction.y = -1.0;
        }
        let mut position = *body.position();
        // If we assume that planet origin is zero
        let q = na::UnitQuaternion::face_towards(&position.translation.vector, &na::Vector3::z());
        position.rotation = na::convert(q * state.orientation);
        body.set_position(position);

        let mut movement_direction_transformed = q.transform_vector(&movement_direction.xzy());
        let altitude = body.position().translation.vector.norm() - 1275620 as f64;
        let player_velocity = body.velocity().linear;
        let up = q.transform_vector(&na::Vector3::new(0.0, 0.0, 1.0));

        if state.fly {
            if state.jump && !state.sit {
                movement_direction_transformed += up;
            }
            body.apply_force(
                0,
                &nphysics3d::math::Force::new(
                    movement_direction_transformed * altitude.abs().max(1.0) - player_velocity,
                    na::zero(),
                ),
                nphysics3d::algebra::ForceType::VelocityChange,
                true,
            );
        } else if handle.on_surface || on_surface {
            if state.jump {
                body.apply_force(
                    0,
                    &nphysics3d::math::Force::new(up * 20000.0, na::zero()),
                    nphysics3d::algebra::ForceType::Force,
                    true,
                );
            }
            movement_direction_transformed *= 8.0;
            body.apply_force(
                0,
                &nphysics3d::math::Force::new(
                    movement_direction_transformed - player_velocity,
                    na::zero(),
                ),
                nphysics3d::algebra::ForceType::VelocityChange,
                true,
            );
        }
    }
}

pub fn spawn(
    world: &mut hecs::World,
    physics: &mut Physics,
    name: String,
    entity_id: shared::EntityId,
) -> hecs::Entity {
    let mut player = hecs::EntityBuilder::new();
    player.add(Transform {
        isometry: na::Isometry3::from_parts(
            na::Translation3::new(0.0, 0.0, 0.0),
            na::UnitQuaternion::from_euler_angles(0.0, 0.0, 3.14),
        ),
        ..Default::default()
    });
    player.add(entity_id);
    player.add(Drawable {
        model: "./assets/models/tree/tree.gltf".to_string(),
        shader: "SIMPLE".to_string(),
    });

    let player_body = physics.add_body(
        RigidBodyDesc::new()
            //.collider(&ColliderDesc::new())
            .mass(40.0)
            .translation(na::Vector3::new(
                996609.65806255,
                -747775.7217986964,
                414785.79067247955,
            ))
            .kinematic_rotations(na::Vector3::new(true, true, true))
            .build(),
        &mut player,
    );
    physics.colliders.insert(
        ColliderDesc::new(ShapeHandle::new(Ball::new(1.5)))
            .material(MaterialHandle::new(BasicMaterial::new(0.0, 0.0)))
            .build(BodyPartHandle(player_body, 0)),
    );
    let ground_sensor_handle = physics.colliders.insert(
        ColliderDesc::new(ShapeHandle::new(Cuboid::new(na::Vector3::new(
            0.75, 0.75, 0.3,
        ))))
        .sensor(true)
        .set_position(na::Isometry3::from_parts(
            na::Translation3::new(0.0, 0.0, -1.5),
            na::UnitQuaternion::identity(),
        ))
        .build(BodyPartHandle(player_body, 0)),
    );
    player.add(Player::new(name, ground_sensor_handle));

    let player_entity = world.spawn(player.build());
    physics.register_entity(player_body, player_entity.clone());
    player_entity
}
