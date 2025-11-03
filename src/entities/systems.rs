use bevy::prelude::*;
use rand::Rng;

use super::components::{Position, Velocity, EntityType, EntityColor, Hunger, BehaviorState, SenseRadius};
use super::behaviors::{Behavior, SeekFood, Sleep, InfluencedWork};

const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
const YELLOW: Color = Color::srgb(1.0, 1.0, 0.0);

fn create_food(pos: Vec2) -> (Position, EntityType, EntityColor, SpriteBundle) {
    (
        Position(pos),
        EntityType::Food,
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
    )
}
fn create_prey(pos: Vec2, velo: Vec2, hunger: f32, sense_radus:f32) -> (Position, Velocity,EntityType, EntityColor,Hunger,BehaviorState, SenseRadius,  SpriteBundle) {
    (
        Position(pos),
        Velocity(velo),
        EntityType::Prey,
        EntityColor(YELLOW),
        Hunger(hunger),
        BehaviorState::Sleep,
        SenseRadius(sense_radus),
        SpriteBundle {
            sprite: Sprite {
                color: YELLOW,
                custom_size: Some(Vec2::new(2.0, 2.0)),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.0)),
            ..default()
        },
    )
}

pub fn setup_entities(mut commands: Commands) {
    // Only spawn default entities if no config was loaded
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let pos = Vec2::new(
            rng.gen_range(0.0..1000.0),
            rng.gen_range(0.0..1000.0),
        );
        let vel = Vec2::new(
            rng.gen_range(-50.0..50.0),
            rng.gen_range(-50.0..50.0),
        );
        commands.spawn(create_prey(pos, vel, rng.gen_range(0.0..50.0), rng.gen_range(70.0..100.0)));
    }
    for _ in 0..200 {
        commands.spawn(create_food(Vec2::new(
            rng.gen_range(0.0..1000.0),
            rng.gen_range(0.0..1000.0),
        )));
    }
    info!("Spawned 1000 Prey and 200 Food entities");
}

pub fn update_behaviors(
    mut query: Query<(Entity, &mut Position, &Hunger, &SenseRadius, &mut Velocity, &mut BehaviorState, &mut Transform)>,
    food_query: Query<(&Position, &EntityType), Without<BehaviorState>>,
    all_entities: Query<(Entity, &Position, &EntityType), (With<EntityType>, Without<BehaviorState>)>,
    time: Res<Time>,
) {
    for (entity, mut position, hunger, sense_radius, mut velocity, mut behavior_state, mut transform) in query.iter_mut() {
        let behavior: Box<dyn Behavior> = match *behavior_state {
            BehaviorState::SeekFood => Box::new(SeekFood),
            BehaviorState::Sleep => Box::new(Sleep),
            BehaviorState::InfluencedWork => Box::new(InfluencedWork),
        };
        let (new_velocity, new_state) = behavior.execute(entity, &position, hunger, &sense_radius, &food_query, &all_entities, &time);
        velocity.0 = new_velocity;
        *behavior_state = new_state;

        // Update position
        position.0 += velocity.0;
        position.0.x = position.0.x.rem_euclid(1000.0);
        position.0.y = position.0.y.rem_euclid(1000.0);
        transform.translation = position.0.extend(0.0);
    }
}

pub fn update_hunger(
    mut query: Query<(&mut Hunger), With<EntityType>>, time: Res<Time>
) {
    for mut hunger in query.iter_mut() {
        hunger.0 += 3.0 * time.delta_seconds();
        hunger.0 = hunger.0.clamp(0.0, 100.0);
        // if hunger.0 > 30.0 {
        // info!("Prey {:?} hungry {:?}",prey, hunger.0);
        // }
    }
}

pub fn eat_food(
    mut commands: Commands,
    mut prey_query: Query<(Entity, &Position, &mut Hunger, &mut BehaviorState, &EntityType), With<EntityType>>,
    food_query: Query<(Entity, &Position, &EntityType), (With<EntityType>, Without<BehaviorState>)>,
) {
    for (prey_entity, prey_pos, mut hunger, mut behavior_state, prey_type) in prey_query.iter_mut() {
        if *behavior_state != BehaviorState::SeekFood || !matches!(prey_type, EntityType::Prey) {
            continue;
        }

        // Find the closest Food entity within 1 unit
        let mut closest_food = None;
        let mut min_distance = f32::MAX;
        for (food_entity, food_pos, food_type) in food_query.iter() {
            if matches!(food_type, EntityType::Food) {
                let distance = prey_pos.0.distance(food_pos.0);
                if distance <= 2.0 && distance < min_distance {
                    min_distance = distance;
                    closest_food = Some(food_entity);
                }
            }
        }

        // Eat the closest Food if found
        if let Some(food_entity) = closest_food {
            commands.entity(food_entity).despawn();
            hunger.0 = (hunger.0 - 50.0).max(0.0);
            if hunger.0 < 20.0 {
                *behavior_state = BehaviorState::Sleep;
            } else {
                *behavior_state = BehaviorState::SeekFood; // Re-seek new Food
            }
            // info!(
            //     "Prey {:?} (type: {:?}) ate Food {:?} (type: Food), hunger now {}, distance: {}",
            //     prey_entity, prey_type, food_entity, hunger.0, min_distance
            // );
        }
    }
}

pub fn starve(
    mut commands: Commands,
    prey_query: Query<(Entity, &Position, &Hunger, &BehaviorState, &EntityType), With<EntityType>>,
    food_query: Query<&EntityType, With<EntityType>>,
) {
    let food_available = food_query.iter().any(|et| matches!(et, EntityType::Food));
    for (prey_entity, position, hunger, behavior_state, entity_type) in prey_query.iter() {
        if hunger.0 >= 100.0 && !food_available && *behavior_state == BehaviorState::SeekFood && matches!(entity_type, EntityType::Prey) {
            commands.entity(prey_entity).despawn();
            commands.spawn(create_food(position.0));
            // info!("Prey {:?} (type: {:?}) starved and died", prey_entity, entity_type);
        }
    }
}