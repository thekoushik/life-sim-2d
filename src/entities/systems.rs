use bevy::prelude::*;
use rand::Rng;

use super::components::{Position, Velocity, Food, Prey, EntityColor, Hunger, BehaviorState, SenseRadius, Brain, Needs, Genes };
// use super::behaviors::{Behavior, SeekFood, Sleep, Wander};

const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
const YELLOW: Color = Color::srgb(1.0, 1.0, 0.0);

fn create_food(pos: Vec2) -> (Position, Food, EntityColor, SpriteBundle) {
    (
        Position(pos),
        Food,
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
fn create_prey(pos: Vec2, velo: Vec2, hunger: f32, sense_radus:f32) -> (Position, Velocity,Prey, EntityColor,Hunger,BehaviorState, SenseRadius,  SpriteBundle) {
    (
        Position(pos),
        Velocity(velo),
        Prey,
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

pub fn game_loop(
    mut commands: Commands,
    mut prey_query: Query<(
        Entity,&mut Position, &mut Hunger, &mut BehaviorState, &SenseRadius, &mut Transform
    ), With<Prey>>,
    food_query: Query<(Entity,&Position), (With<Food>, Without<BehaviorState>)>,
    time: Res<Time>
) {
    for (
        prey_entity,
        mut prey_pos,
        mut hunger,
        mut behavior_state,
        sense_radius,
        mut transform
    ) in prey_query.iter_mut() {
        hunger.0 += 3.0 * time.delta_seconds();
        hunger.0 = hunger.0.clamp(0.0, 100.0);

        // Find the closest Food entity within 1 unit
        let mut closest_food = None;
        let mut nearest_food_pos = None;
        let mut min_distance = sense_radius.0;
        for (food_entity, food_pos) in food_query.iter() {
            let distance = prey_pos.0.distance(food_pos.0);
            if distance <= 2.0 {
                closest_food = Some(food_entity);
                break;
            } else if distance < min_distance {
                min_distance = distance;
                nearest_food_pos = Some(food_pos.0);
            }
        }

        // Eat the closest Food if found
        if let Some(food_entity) = closest_food {
            commands.entity(food_entity).despawn();
            hunger.0 = (hunger.0 - 50.0).max(0.0);
            // info!(
            //     "Prey {:?} (type: {:?}) ate Food {:?} (type: Food), hunger now {}, distance: {}",
            //     prey_entity, prey_type, food_entity, hunger.0, min_distance
            // );
        } else if hunger.0 >= 100.0 {
            commands.entity(prey_entity).despawn();
            commands.spawn(create_food(prey_pos.0));
        } else if let Some(food_pos) = nearest_food_pos {
            // Move toward nearest food
            let direction = (food_pos - prey_pos.0).normalize_or_zero();
            // more hungry = more speed
            let move_distance = (10.0 * hunger.0 / 100.0) * time.delta_seconds(); // Move at 10 units/s
            // *behavior_state = BehaviorState::SeekFood;
            prey_pos.0 += direction * move_distance;
            // Update position
            prey_pos.0.x = prey_pos.0.x.rem_euclid(1000.0);
            prey_pos.0.y = prey_pos.0.y.rem_euclid(1000.0);
            transform.translation = prey_pos.0.extend(0.0);
        }
        if hunger.0 < 20.0 {
            *behavior_state = BehaviorState::Sleep;
        } else {
            *behavior_state = BehaviorState::SeekFood; // Re-seek new Food
        }
    }
}

// fn wandering_system(
//     mut query: Query<(&Genes, &mut Brain, &Transform)>,
//     time: Res<Time>,
// ) {
//     for (genes, mut brain, transform) in &mut query {
//         brain.time_since_last_target += time.delta_seconds();

//         // Change wander target periodically based on curiosity
//         let change_interval = 3.0.lerp(12.0, 1.0 - genes.curiosity);
//         if brain.time_since_last_target > change_interval {
//             brain.time_since_last_target = 0.0;
//             let angle = rand::random::<f32>() * std::f32::consts::TAU;
//             let distance = rand::random::<f32>() * genes.wander_radius;
//             brain.target = Some(transform.translation.truncate() + Vec2::from_angle(angle) * distance);
//         }
//     }
// }

// fn perception_system(
//     mut query: Query<(&Transform, &Genes, &mut Perception)>,
//     foods: Query<&Transform, With<Food>>,
//     predators: Query<&Transform, With<Predator>>,
// ) {
//     for (transform, genes, mut perception) in &mut query {
//         perception.visible_food.clear();
//         perception.visible_predators.clear();

//         let pos = transform.translation.truncate();

//         for food in foods.iter() {
//             let dist = pos.distance(food.translation.truncate());
//             if dist < genes.vision_range {
//                 perception.visible_food.push(food.translation.truncate());
//             }
//         }

//         // similar for predators
//     }
// }

// fn decision_system(
//     mut query: Query<(&Genes, &Needs, &Perception, &mut Brain)>
// ) {
//     for (genes, needs, perception, mut brain) in &mut query {
//         if needs.fear > genes.panic_threshold && !perception.visible_predators.is_empty() {
//             brain.state = State::Flee;
//         } else if needs.hunger > 0.7 && !perception.visible_food.is_empty() {
//             brain.state = State::SeekFood;
//         } else if needs.energy < 0.3 && rand::random::<f32>() < genes.laziness {
//             brain.state = State::Sleep;
//         } else {
//             brain.state = State::Wander;
//         }
//     }
// }

// fn mutate_genes(parent: &Genes) -> Genes {
//     let mut rng = rand::thread_rng();
//     let mutate = |v: f32| (v + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);
//     Genes {
//         curiosity: mutate(parent.curiosity),
//         boldness: mutate(parent.boldness),
//         greed: mutate(parent.greed),
//         laziness: mutate(parent.laziness),
//         panic_threshold: mutate(parent.panic_threshold),
//         aggression: mutate(parent.aggression),
//         vision_range: (parent.vision_range + rng.gen_range(-2.0..2.0)).max(1.0),
//         smell_range: (parent.smell_range + rng.gen_range(-2.0..2.0)).max(1.0),
//         wander_radius: (parent.wander_radius + rng.gen_range(-5.0..5.0)).max(1.0),
//         max_speed: (parent.max_speed + rng.gen_range(-0.2..0.2)).max(0.1),
//     }
// }
