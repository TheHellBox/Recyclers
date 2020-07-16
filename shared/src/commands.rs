use crate::components::*;
use crate::EntityId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientInfo {
    pub name: String,
}

// TODO: Send layer configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub character_id: u32,
    pub tickrate: u8,
    pub planet_seed: u16,
    pub planet_radius: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tick {
    pub spawns: Vec<(EntityId, Vec<Component>)>,
    // I hate the fact that we utilize f64s for position updates. This just makes every other netcode optimization dull
    pub positions: Vec<(EntityId, na::Isometry3<f64>)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Component {
    Transform(Transform),
    Drawable(Drawable),
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct ClientCommand {
    pub movement_direction: na::Vector2<i8>,
    // NOTE: I can change f32 to i16/i8. Not sure if it's needed though
    pub orientation: na::UnitQuaternion<f64>,
    pub fly: bool,
    pub jump: bool,
    pub run: bool,
    pub sit: bool,
    pub pickup: bool,
    pub prop_spawn: Option<u8>,
}
