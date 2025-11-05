use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy::utils::HashMap;
use bevy::math::IVec2;
use crate::helpers::util::{GREEN, YELLOW, GRAY};

#[derive(Resource, Default)]
pub struct SpatialGrid {
    pub buckets: HashMap<IVec2, Vec<Entity>>,
    pub cell_size: f32,
}

#[derive(Resource)]
pub struct SimulationSpeed(pub f32);

#[derive(Component, Clone)]
pub struct Genes {
    // personality traits (0.0 - 1.0 range)
    pub curiosity: f32,        // how often it changes wander target
    // pub boldness: f32,         // how close it dares approach predators
    pub greed: f32,            // how far it goes for food or wants to eat
    pub laziness: f32,         // prefers resting vs exploring
    // pub panic_threshold: f32,  // how easily it flees
    // pub aggression: f32,       // relevant for predator

    // sense and physical limits
    pub vision_range: f32,
    // pub smell_range: f32,
    pub wander_radius: f32,
    pub max_speed: f32,
    pub bite_size: f32, // how much food it can eat at once
    pub hunger_rate: f32, // how much hunger it gains per second
}

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct Position(pub Vec2);

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Prey;

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Food;


#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct FoodAmount(pub f32); // How much food is left in the food entity

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Predator;

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct LivingEntity;

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct WorldObject;

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Corpse;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct EntityColor(pub Color);

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct Hunger(pub f32); // Hunger for Prey (0.0 = full, 100.0 = starving)

#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub enum BehaviorState {
    SeekFood,
    Sleep,
    // Flee,
    Wander,
}

#[derive(Component, Default, Clone)]
pub struct Perception {
    pub target_food: Option<Entity>,
    pub visible_predators: Vec<Entity>,
    // pub nearby_predator: bool,
    pub time_since_last_sense: f32,
    pub neighbors: Vec<Vec2>,
    pub target: Option<Vec2>,
    pub time_since_last_target: f32,
}

// #[derive(Component)]
// pub struct Brain {
//     // pub state: BehaviorState,
//     pub target: Option<Vec2>,
//     pub time_since_last_target: f32,
// }

#[derive(Component)]
pub struct Needs {
    pub hunger: f32,
    pub energy: f32,
    pub fear: f32,
}


pub fn create_food(pos: Vec2, amount: f32) -> (Position, Food, WorldObject, EntityColor, SpriteBundle, FoodAmount) {
    (
        Position(pos),
        Food,
        WorldObject,
        EntityColor(GREEN),
        SpriteBundle {
            sprite: Sprite {
                color: GREEN,
                custom_size: Some(Vec2::new(2.0, 2.0)), // Smaller radius ~3
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.0)),
            ..default()
        },
        FoodAmount(amount),
    )
}
pub fn create_prey(pos: Vec2, hunger: f32, gene: Genes) -> (Position, Prey,WorldObject, LivingEntity, EntityColor,Hunger,BehaviorState, Genes,  SpriteBundle, Perception) {
    (
        Position(pos),
        Prey,
        WorldObject,
        LivingEntity,
        EntityColor(YELLOW),
        Hunger(hunger),
        BehaviorState::Wander,
        gene,
        // SenseRadius(sense_radus),
        SpriteBundle {
            sprite: Sprite {
                color: YELLOW,
                custom_size: Some(Vec2::new(2.0, 2.0)),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.0)),
            ..default()
        },
        Perception::default(),
    )
}
pub fn create_corpse(pos: Vec2) -> (Position, Corpse, WorldObject, SpriteBundle) {
    (
        Position(pos),
        Corpse,
        WorldObject,
        SpriteBundle {
            sprite: Sprite {
                color: GRAY,
                custom_size: Some(Vec2::new(2.0, 2.0)),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.0)),
            ..default()
        },
    )
}