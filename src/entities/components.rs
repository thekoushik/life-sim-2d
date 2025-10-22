use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct Position(pub Vec2);

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Velocity(pub Vec2);

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum EntityType {
    Prey,
    Food, // New variant for food entities
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct EntityColor(pub Color);

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Hunger(pub f32); // Hunger for Prey (0.0 = full, 100.0 = starving)

#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub enum BehaviorState {
    SeekFood,
    Sleep,
    InfluencedWork,
}
