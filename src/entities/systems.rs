use bevy::prelude::*;
use rand::Rng;

use super::components::{Position, Velocity, Food, Prey, EntityColor, Hunger, BehaviorState, SenseRadius, Needs, Genes, SpatialGrid, Living, Perception,Predator };

const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
const YELLOW: Color = Color::srgb(1.0, 1.0, 0.0);
const NEIGHBOR_CELLS:[IVec2;9] = [
    IVec2::new(-1, -1), IVec2::new(-1, 0), IVec2::new(-1, 1),
    IVec2::new(0, -1),  IVec2::new(0, 0),  IVec2::new(0, 1),
    IVec2::new(1, -1),  IVec2::new(1, 0),  IVec2::new(1, 1),
];
const NEARBY_AVOIDANCE_DISTANCE: f32 = 5.0;
const COLLISION_RADIUS: f32 = 4.0;

fn create_food(pos: Vec2) -> (Position, Food, Living, EntityColor, SpriteBundle) {
    (
        Position(pos),
        Food,
        Living,
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
fn create_prey(pos: Vec2, velo: Vec2, hunger: f32, gene: Genes) -> (Position, Velocity,Prey,Living, EntityColor,Hunger,BehaviorState, Genes,  SpriteBundle, Perception) {
    (
        Position(pos),
        Velocity(velo),
        Prey,
        Living,
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
        commands.spawn(create_prey(
            pos,
            vel,
            rng.gen_range(0.0..30.0),
            Genes {
                vision_range: rng.gen_range(70.0..100.0),
                laziness: rng.gen_range(0.0..1.0),
                greed: rng.gen_range(0.0..1.0),
                curiosity: rng.gen_range(0.0..1.0),
                wander_radius: rng.gen_range(200.0..500.0),
                // aggression: 0.0,
                // boldness: 0.0,
                // max_speed: 10.0,
                // panic_threshold: 0.0,
                // smell_range: 0.0,
            }
        ));
    }
    for _ in 0..200 {
        commands.spawn(create_food(Vec2::new(
            rng.gen_range(0.0..1000.0),
            rng.gen_range(0.0..1000.0),
        )));
    }
    info!("Spawned 1000 Prey and 200 Food entities");
}

pub fn update_grid_system(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Transform), With<Living>>,
) {
    grid.buckets.clear();
    for (entity, transform) in query.iter() {
        let pos = transform.translation.truncate();
        let cell = IVec2::new(
            (pos.x / grid.cell_size).floor() as i32,
            (pos.y / grid.cell_size).floor() as i32,
        );
        grid.buckets.entry(cell).or_default().push(entity);
    }
}

pub fn perception_scan_system(
    grid: Res<SpatialGrid>,
    mut query: Query<(Entity, &Transform, &Genes, &mut Perception, &BehaviorState), With<Prey>>,
    lookup_query: Query<&Transform, With<Living>>,
    food_query: Query<Entity, With<Food>>,
    predator_query: Query<Entity, With<Predator>>,
    time: Res<Time>
) {
    for (
        entity, transform, genes, mut perception, behavior_state
    ) in query.iter_mut() {
        perception.time_since_last_sense += time.delta_seconds();
        perception.time_since_last_target += time.delta_seconds();
        let mut skip_sense = false;

        if perception.time_since_last_sense < (genes.laziness * 5.0) || *behavior_state == BehaviorState::Sleep {
            // too lazy or sleeping would not be able to see nearby entities
            // but should know the position of nearby entities so we can avoid them
            skip_sense = true;
        }

        perception.neighbors.clear();
        if !skip_sense {
            perception.target_food = None;
            perception.visible_predators.clear();
            
            if *behavior_state == BehaviorState::Wander {
                let change_interval = 3.0.lerp(12.0, 1.0 - genes.curiosity);
                if perception.time_since_last_target > change_interval {
                    perception.time_since_last_target = 0.0;
                    let angle = rand::random::<f32>() * std::f32::consts::TAU;
                    let distance = rand::random::<f32>() * genes.wander_radius;
                    perception.target = Some(transform.translation.truncate() + Vec2::from_angle(angle) * distance);
                }
            } else {
                perception.target = None;
            }
        }

        let pos = transform.translation.truncate();
        let cell = IVec2::new(
            (pos.x / grid.cell_size).floor() as i32,
            (pos.y / grid.cell_size).floor() as i32,
        );
        let mut visible_food: Vec<(Entity, f32)> = Vec::new();
        for offset in NEIGHBOR_CELLS {
            if let Some(entities) = grid.buckets.get(&(cell + offset)) {
                for &other in entities {
                    if other == entity { continue; }
                    let Ok(other_transform) = lookup_query.get(other) else { continue; };

                    let other_pos = other_transform.translation.truncate();
                    let dist = pos.distance(other_pos);

                    if dist < NEARBY_AVOIDANCE_DISTANCE { // very close position occupied by others
                        perception.neighbors.push(other_pos);
                    }
                    if !skip_sense {
                        // if other is food
                        if food_query.get(other).is_ok() && dist < genes.vision_range {
                            visible_food.push((other, dist));
                        }
                        // if other is predator
                        if predator_query.get(other).is_ok() && dist < genes.vision_range {
                            perception.visible_predators.push(other);
                        }
                    }
                }
            }
        }
        if !visible_food.is_empty() {
            if rand::random::<f32>() < 0.8 {
                // 80% chance to prefer closer target
                visible_food.sort_by(|a, b| a.1.total_cmp(&b.1));
                perception.target_food = Some(visible_food[0].0);
            } else {
                // 20% chance to make a "mistake" and pick a random one
                let idx = rand::random::<usize>() % visible_food.len();
                perception.target_food = Some(visible_food[idx].0);
            }
        }
        // info!("Entity {:?} sees {} food", entity, perception.visible_food.len() );
    }
}

// pub fn decision_system(
//     mut query: Query<(&mut Brain, &Needs, &Perception, &Genes)>,
// ) {
//     for (mut brain, needs, perception, genes) in query.iter_mut() {
//         brain.state = if !perception.visible_predators.is_empty() && needs.fear > genes.panic_threshold {
//             State::Flee
//         } else if needs.hunger > 0.7 && !perception.visible_food.is_empty() {
//             State::SeekFood
//         } else if needs.energy < 0.3 {
//             State::Sleep
//         } else {
//             State::Wander
//         };
//     }
// }

pub fn game_loop(
    mut commands: Commands,
    mut prey_query: Query<(
        Entity,&mut Position, &mut Hunger, &mut BehaviorState, &Genes,
        &mut Transform,
        &Perception
    ), With<Prey>>,
    food_query: Query<(Entity,&Position), (With<Food>, Without<BehaviorState>)>,
    time: Res<Time>
) {
    for (
        prey_entity,
        mut prey_pos,
        mut hunger,
        mut behavior_state,
        genes,
        mut transform,
        perception
    ) in prey_query.iter_mut() {
        hunger.0 += 0.5 * time.delta_seconds();
        hunger.0 = hunger.0.clamp(0.0, 100.0);

        let mut nearest_food_pos = None;
        if let Some(food) = perception.target_food {
            if let Ok((food_entity, food_pos)) = food_query.get(food) {
                let distance = prey_pos.0.distance(food_pos.0);
                if distance <= 2.0 {
                    // closest_food = Some(food_entity);
                    commands.entity(food_entity).despawn();
                    hunger.0 = (hunger.0 - 50.0).max(0.0);
                } else {
                    nearest_food_pos = Some(food_pos.0);
                }
            }
        }
        let mut desired_velocity = Vec2::ZERO;
        if hunger.0 >= 100.0 {
            commands.entity(prey_entity).despawn();
            commands.spawn(create_food(prey_pos.0));
        } else if let Some(food_pos) = nearest_food_pos {
            // Move toward nearest food
            let direction = (food_pos - prey_pos.0).normalize_or_zero();
            // more hungry = more speed
            let move_distance = (10.0 * hunger.0 / 100.0) * time.delta_seconds(); // Move at 10 units/s
            desired_velocity = direction * move_distance;
        } else if let Some(target) = perception.target {
            let direction = (target - prey_pos.0).normalize_or_zero();
            let move_distance = 10.0 * time.delta_seconds(); // Move at wander speed
            desired_velocity = direction * move_distance;
        }
        let mut avoidance_force = Vec2::ZERO;
        for &neighbor_pos in perception.neighbors.iter() {
            // its the food I am going to eat, so I should not avoid it
            if let Some(food_pos) = nearest_food_pos {
                if neighbor_pos == food_pos {
                    continue;
                }
            }
            let distance = prey_pos.0.distance(neighbor_pos);
            let direction = (prey_pos.0 - neighbor_pos).normalize_or_zero();
            let strength = (COLLISION_RADIUS - distance) / COLLISION_RADIUS; // Stronger when closer
            avoidance_force += direction * strength * 50.0 * time.delta_seconds();
        }

        prey_pos.0 += desired_velocity + avoidance_force;
        // Update position
        prey_pos.0.x = prey_pos.0.x.rem_euclid(1000.0);
        prey_pos.0.y = prey_pos.0.y.rem_euclid(1000.0);
        transform.translation = prey_pos.0.extend(0.0);
        if hunger.0 > 50.0 || (hunger.0 < 80.0 && genes.greed > 0.5) {
            *behavior_state = BehaviorState::SeekFood; // Re-seek new Food
        } else if genes.laziness > 0.5 {
            *behavior_state = BehaviorState::Sleep;
        } else {
            *behavior_state = BehaviorState::Wander;
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
