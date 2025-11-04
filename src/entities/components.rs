use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone)]
pub struct Genes {
    // personality traits (0.0 - 1.0 range)
    pub curiosity: f32,        // how often it changes wander target
    pub boldness: f32,         // how close it dares approach predators
    pub greed: f32,            // how far it goes for food
    pub laziness: f32,         // prefers resting vs exploring
    pub panic_threshold: f32,  // how easily it flees
    pub aggression: f32,       // relevant for predator

    // sense and physical limits
    pub vision_range: f32,
    pub smell_range: f32,
    pub wander_radius: f32,
    pub max_speed: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct Position(pub Vec2);

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Velocity(pub Vec2);

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SenseRadius(pub f32);

// #[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
// pub enum EntityType {
//     Prey,
//     Food, // New variant for food entities
// }

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Prey;

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Food;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct EntityColor(pub Color);

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Hunger(pub f32); // Hunger for Prey (0.0 = full, 100.0 = starving)

#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub enum BehaviorState {
    SeekFood,
    Sleep,
    // Flee,
    Wander,
}

#[derive(Component, Default)]
struct Perception {
    visible_food: Vec<Entity>,
    visible_predators: Vec<Entity>,
}

#[derive(Component)]
pub struct Brain {
    pub state: BehaviorState,
    pub target: Option<Vec2>,
    pub time_since_last_target: f32,
}

#[derive(Component)]
pub struct Needs {
    pub hunger: f32,
    pub energy: f32,
    pub fear: f32,
}

