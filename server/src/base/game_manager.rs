use crate::base::components::physics::PhysicsBody;
use crate::base::player::Player;
use crate::base::props::PropData;
use crate::base::systems::physics::Physics;

use hecs::Entity;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use shared::{commands::Component, components::*, EntityId};
use std::collections::HashMap;

pub struct GameManager {
    pub world: hecs::World,
    pub physics: Physics,
    pub entity_ids: HashMap<EntityId, Entity>,
    pub props: HashMap<usize, PropData>,
    spawns: Vec<Entity>,
    rng: SmallRng,
}

impl GameManager {
    pub fn new() -> Self {
        GameManager {
            world: hecs::World::new(),
            physics: Physics::new(),
            entity_ids: HashMap::with_capacity(2048),
            spawns: Vec::with_capacity(256),
            rng: SmallRng::from_entropy(),
            props: HashMap::new(),
        }
    }
    pub fn step(
        &mut self,
    ) -> (
        Vec<(EntityId, Vec<Component>)>,
        Vec<(EntityId, na::Isometry3<f64>)>,
    ) {
        self.physics.run(&mut self.world);

        let mut props = vec![];
        self.manage_pickables();
        self.manage_welds();
        for (_entity, (player, physics_body)) in
            self.world.query::<(&mut Player, &PhysicsBody)>().iter()
        {
            if player.state.is_none() {
                continue;
            };
            let planet_handle = self.physics.planet_handle.clone();
            player.walk(&mut self.physics, physics_body, planet_handle);
            if let Some(prop) = player.state.unwrap().prop_spawn {
                props.push((prop, physics_body.clone()));
            }
            player.state = None;
        }
        for (prop, owner) in props {
            self.spawn_prop(&owner, prop as usize);
        }
        let mut new_spawns = Vec::with_capacity(self.spawns.len());
        let mut positions = vec![];
        for entity in self.spawns.drain(..) {
            let entity_id = *self.world.get::<EntityId>(entity).unwrap();
            new_spawns.push((entity_id, pull_components(&self.world, entity)));
        }
        for (_entity, (&id, &transform)) in &mut self.world.query::<(&EntityId, &Transform)>() {
            positions.push((id, transform.isometry));
        }
        (new_spawns, positions)
    }
    pub fn spawn_player(&mut self, info: shared::commands::ClientInfo) -> (EntityId, hecs::Entity) {
        let id = self.new_id();
        let player = crate::base::player::spawn(&mut self.world, &mut self.physics, info.name, id);
        self.spawn(player);
        (id, player)
    }
    pub fn spawn(&mut self, entity: Entity) {
        self.spawns.push(entity);
        let id = self.world.get::<EntityId>(entity);
        if let Ok(id) = id {
            self.entity_ids.insert(*id, entity);
        }
    }
    pub fn snapshot(&mut self) -> Vec<(EntityId, Vec<Component>)> {
        let mut entities = vec![];
        for (entity, &id) in &mut self.world.query::<&EntityId>() {
            entities.push((id, pull_components(&self.world, entity)));
        }
        entities
    }
    pub fn new_id(&mut self) -> EntityId {
        loop {
            let id = self.rng.gen();
            if self.entity_ids.contains_key(&id) {
                continue;
            }
            return id;
        }
    }
}

// NOTE: It doesn't looks like the best way to do it. But it'll work for now, we don't have lots of components
// Maybe I can make it into macro or smthng
fn pull_components(world: &hecs::World, entity: Entity) -> Vec<Component> {
    let mut components = Vec::new();
    if let Ok(x) = world.get::<Transform>(entity) {
        components.push(Component::Transform((*x).clone()));
    }
    if let Ok(x) = world.get::<Drawable>(entity) {
        components.push(Component::Drawable((*x).clone()));
    }
    components
}
