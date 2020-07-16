extern crate nalgebra as na;

pub mod commands;
pub mod components;
pub mod network;
pub mod planet;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u32);

impl From<u32> for EntityId {
    fn from(x: u32) -> Self {
        Self(x)
    }
}

impl From<EntityId> for u32 {
    fn from(x: EntityId) -> u32 {
        x.0
    }
}

impl Distribution<EntityId> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EntityId {
        EntityId(rng.gen())
    }
}
