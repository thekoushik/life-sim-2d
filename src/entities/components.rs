use crate::helpers::util::{GRAY, GREEN, YELLOW};
use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::utils::HashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};

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
    pub sociality: f32, // 0.0 = introvert, 1.0 = extrovert
    pub curiosity: f32, // how often it changes wander target
    // pub boldness: f32,         // how close it dares approach predators
    pub greed: f32,    // how far it goes for food or wants to eat
    pub laziness: f32, // prefers resting vs exploring
    // pub panic_threshold: f32,  // how easily it flees
    // pub aggression: f32,       // relevant for predator

    // sense and physical limits
    pub vision_range: f32,
    pub wander_radius: f32,
    pub max_speed: f32,
    pub bite_size: f32,   // how much food it can eat at once
    pub hunger_rate: f32, // how much hunger it gains per second
    pub max_age: f32,     // how long the entity can live

    pub gender: bool,             // true = female, false = male
    pub max_offspring_count: u32, // how many offspring the entity can produce
    pub can_produce_food: bool,   // whether the entity can produce food
}

impl Default for Genes {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let gender = rng.gen_bool(0.5);
        Self {
            sociality: rng.gen_range(0.0..1.0),
            vision_range: rng.gen_range(300.0..500.0),
            laziness: rng.gen_range(0.0..1.0),
            greed: rng.gen_range(0.0..1.0),
            curiosity: rng.gen_range(0.0..1.0),
            wander_radius: rng.gen_range(300.0..600.0),
            bite_size: rng.gen_range(1.0..10.0),
            max_speed: rng.gen_range(5.0..10.0),
            hunger_rate: rng.gen_range(0.5..1.0),
            max_age: rng.gen_range(100.0..300.0),
            gender: gender,
            max_offspring_count: if gender { rng.gen_range(1..10) } else { 0 },
            can_produce_food: if gender { rng.gen_bool(0.5) } else { false },
            // aggression: 0.0,
            // boldness: 0.0,
            // panic_threshold: 0.0,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Age(pub f32);

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

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CorpseState {
    pub flesh_amount: f32, // how much flesh is left in the corpse
    pub decay_rate: f32,   // how fast the corpse decays
    pub decay_timer: f32, // how long the corpse has been decaying, will be deleted when it reaches 0
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct EntityColor(pub Color);

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
    pub nearby_corpses: Vec<(Vec2, f32)>, // (position, stench_radius)
    pub nearby_mates: Vec<Entity>,
}

// #[derive(Component)]
// pub struct Brain {
//     // pub state: BehaviorState,
//     pub target: Option<Vec2>,
//     pub time_since_last_target: f32,
// }

#[derive(Component)]
pub struct Needs {
    pub fear: f32,   // high fear = slower movement
    pub sanity: f32, // low sanity = more aggressive

    pub hunger: f32, // hunger should influence sanity
    pub energy: f32, // low energy = slower movement

    // reproduction related
    pub mother: Option<Entity>,       // the entity it is mother of
    pub pregnancy_timer: f32,         // how long the entity has been pregnant
    pub pregnant: bool,               // whether the entity is pregnant
    pub mating_timer: f32,            // how long is it staying with a partner before mating
    pub partner: Option<Entity>,      // the entity it is mating with
    pub partner_genes: Option<Genes>, // the genes of the partner
    pub partner_timer: f32,           // how long stay together being partners
    pub mate_ready: bool,             // whether the entity is ready to mate
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            fear: 0.0,
            sanity: 1.0,
            hunger: 0.0,
            energy: 1.0,
            mother: None,
            pregnancy_timer: 0.0,
            pregnant: false,
            mating_timer: 0.0,
            partner: None,
            partner_genes: None,
            partner_timer: 0.0,
            mate_ready: false,
        }
    }
}
//TODO: create species config with a genetic config and a name
// and entities can vary a little bit from the genetic config
// pub struct Species {
//     pub name: String,
//     pub genetic_min: Genes,
//     pub genetic_max: Genes,
// }

pub fn create_food(
    pos: Vec2,
    amount: f32,
) -> (
    Position,
    Food,
    WorldObject,
    EntityColor,
    SpriteBundle,
    FoodAmount,
) {
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
pub fn create_prey(
    pos: Vec2,
    gene: Genes,
) -> (
    Position,
    Prey,
    WorldObject,
    LivingEntity,
    EntityColor,
    BehaviorState,
    Genes,
    SpriteBundle,
    Perception,
    Age,
    Needs,
) {
    (
        Position(pos),
        Prey,
        WorldObject,
        LivingEntity,
        EntityColor(YELLOW),
        BehaviorState::Wander,
        gene,
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
        Age(0.0),
        Needs::default(),
    )
}
pub fn create_corpse(
    pos: Vec2,
    flesh_amount: f32,
) -> (Position, Corpse, WorldObject, SpriteBundle, CorpseState) {
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
        CorpseState {
            flesh_amount: flesh_amount,
            decay_rate: 1.0,
            decay_timer: 100.0,
        },
    )
}

pub fn mutate_genes(mother: &Genes, father: &Genes) -> Genes {
    let mut rng = rand::thread_rng();
    let mut new_gene = mother.clone();
    new_gene.sociality = (mother.sociality + father.sociality) / 2.0;
    new_gene.vision_range = (mother.vision_range + father.vision_range) / 2.0;
    new_gene.wander_radius = (mother.wander_radius + father.wander_radius) / 2.0;
    new_gene.bite_size = (mother.bite_size + father.bite_size) / 2.0;
    new_gene.max_speed = (mother.max_speed + father.max_speed) / 2.0;
    new_gene.hunger_rate = (mother.hunger_rate + father.hunger_rate) / 2.0;
    new_gene.max_age = (mother.max_age + father.max_age) / 2.0;
    new_gene.gender = rng.gen_bool(0.5);
    new_gene.max_offspring_count = (mother.max_offspring_count + father.max_offspring_count) / 2;
    new_gene.can_produce_food = mother.can_produce_food || father.can_produce_food;
    new_gene
}
