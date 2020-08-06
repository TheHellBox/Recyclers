use crate::base::components::PhysicsBody;
use crate::base::game_manager::GameManager;
use crate::base::player::Player;
use ncollide3d::query::{Ray, RayCast};
use nphysics3d::object::{Body, BodyPartHandle, RigidBody};
use shared::components::Transform;

pub struct PickAble {
    pub owner: Option<hecs::Entity>,
}

impl GameManager {
    pub fn manage_pickables(&mut self) {
        for (_, (player, player_transform)) in
            self.world.query::<(&mut Player, &Transform)>().iter()
        {
            if let Some(state) = player.state {
                if !state.pickup {
                    continue;
                }
                if player.picked_object.is_some() {
                    player.picked_object = None;
                    continue;
                }
                let ray = Ray::new(
                    player_transform.isometry.translation.vector.into(),
                    player_transform.isometry.rotation * -na::Vector3::z() * 5.0,
                );
                for (entity, (_pickable, pickable_body)) in
                    self.world.query::<(&PickAble, &PhysicsBody)>().iter()
                {
                    let collider = self
                        .physics
                        .colliders
                        .get_mut(pickable_body.handle)
                        .unwrap();
                    let body = self
                        .physics
                        .bodies
                        .get_mut(pickable_body.handle)
                        .unwrap()
                        .downcast_mut::<RigidBody<f64>>()
                        .unwrap();
                    if collider
                        .shape()
                        .toi_with_ray(body.position(), &ray, 5.0, false)
                        .is_some()
                    {
                        println!("Picked up");
                        player.picked_object = Some(entity);
                    }
                }
            }
        }
        for (entity, (player, player_transform)) in
            self.world.query::<(&mut Player, &Transform)>().iter()
        {
            if player.picked_object.is_none() {
                continue;
            }
            let pickable_body = self
                .world
                .get::<PhysicsBody>(player.picked_object.unwrap())
                .unwrap();
            let player_body = self.world.get::<PhysicsBody>(entity).unwrap();
            let player_velocity = self
                .physics
                .bodies
                .get(player_body.handle)
                .unwrap()
                .downcast_ref::<RigidBody<f64>>()
                .unwrap()
                .velocity()
                .linear;
            let body = self
                .physics
                .bodies
                .get_mut(pickable_body.handle)
                .unwrap()
                .downcast_mut::<RigidBody<f64>>()
                .unwrap();
            let target_position = player_transform.isometry.translation.vector
                + player_transform.isometry.rotation * -na::Vector3::z() * 4.0;
            let vector = target_position - body.position().translation.vector;
            let velocity = body.velocity().linear;
            if vector.norm() > 4.0 {
                player.picked_object = None;
            }
            let x = player_transform
                .isometry
                .rotation
                .axis()
                .unwrap()
                .normalize()
                .cross(&body.position().rotation.axis().unwrap().normalize());
            let theta = x.norm().asin();
            let w = x.normalize() * theta;
            body.apply_force(
                0,
                &nphysics3d::math::Force::new(
                    player_velocity + vector * 5.0
                        - velocity.normalize() * velocity.norm().min(10.0),
                    w,
                ),
                nphysics3d::algebra::ForceType::Impulse,
                true,
            );
        }
    }
    pub fn manage_welds(&mut self) {
        for (_, player) in self.world.query::<(&mut Player)>().iter() {
            if let Some(state) = player.state {
                if !state.sit {
                    continue;
                }
                if player.picked_object.is_none() {
                    continue;
                }
                let picked_object = player.picked_object.unwrap();
                let picked_body_handle = self.world.get::<PhysicsBody>(picked_object).unwrap();
                let picked_transform = self.world.get::<Transform>(picked_object).unwrap();

                for (coll_handle, body_handle) in &picked_body_handle.collides_with {
                    player.picked_object = None;
                    let other_entity = self.physics.entities.get(&body_handle).unwrap();
                    if self.world.get::<PickAble>(*other_entity).is_err() {
                        continue;
                    }
                    let other_transform = self.world.get::<Transform>(*other_entity).unwrap();

                    let middle_vec = picked_transform
                        .isometry
                        .translation
                        .vector
                        .lerp(&other_transform.isometry.translation.vector, 0.5);
                    let middle_r = picked_transform
                        .isometry
                        .rotation
                        .slerp(&other_transform.isometry.rotation, 0.5);
                    let middle =
                        na::Isometry3::from_parts(na::Translation3::from(middle_vec), middle_r);
                    let anchor_1 = picked_transform.isometry.inverse() * middle;
                    let anchor_2 = other_transform.isometry.inverse() * middle;

                    let constraint = nphysics3d::joint::FixedConstraint::new(
                        BodyPartHandle(picked_body_handle.handle, 0),
                        BodyPartHandle(*body_handle, 0),
                        anchor_1.translation.vector.into(),
                        anchor_1.rotation,
                        anchor_2.translation.vector.into(),
                        anchor_2.rotation,
                    );
                    self.physics.joint_constraints.insert(constraint);
                }
            }
        }
    }
}
