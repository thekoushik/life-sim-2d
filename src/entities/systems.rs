use bevy::prelude::*;
use rand::Rng;

use super::components::{Position, Velocity, EntityType, EntityColor, Hunger, BehaviorState};
use super::behaviors::{Behavior, SeekFood, Sleep, InfluencedWork};

const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
const YELLOW: Color = Color::srgb(1.0, 1.0, 0.0);

pub fn setup_entities(mut commands: Commands) {
    // Only spawn default entities if no config was loaded
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let pos = Vec2::new(
            rng.gen_range(0.0..1000.0),
            rng.gen_range(0.0..1000.0),
        );
        let vel = Vec2::new(
            rng.gen_range(-50.0..50.0),
            rng.gen_range(-50.0..50.0),
        );
        commands.spawn((
            Position(pos),
            Velocity(vel),
            EntityType::Prey,
            EntityColor(GREEN),
            Hunger(0.0),
            BehaviorState::Sleep,
            SpriteBundle {
                sprite: Sprite {
                    color: GREEN,
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                transform: Transform::from_translation(pos.extend(0.0)),
                ..default()
            },
        ));
    }
    for _ in 0..50 {
        let pos = Vec2::new(
            rng.gen_range(0.0..1000.0),
            rng.gen_range(0.0..1000.0),
        );
        commands.spawn((
            Position(pos),
            EntityType::Food,
            EntityColor(YELLOW),
            SpriteBundle {
                sprite: Sprite {
                    color: YELLOW,
                    custom_size: Some(Vec2::new(1.0, 1.0)), // Smaller radius ~3
                    ..default()
                },
                transform: Transform::from_translation(pos.extend(0.0)),
                ..default()
            },
        ));
    }
    info!("Spawned 100 Prey and 50 Food entities");
}

pub fn update_behaviors(
    mut query: Query<(Entity, &mut Position, &Hunger, &mut Velocity, &mut BehaviorState, &mut Transform)>,
    food_query: Query<(&Position, &EntityType), Without<BehaviorState>>,
    time: Res<Time>,
) {
    for (entity, mut position, hunger, mut velocity, mut behavior_state, mut transform) in query.iter_mut() {
        let behavior: Box<dyn Behavior> = match *behavior_state {
            BehaviorState::SeekFood => Box::new(SeekFood),
            BehaviorState::Sleep => Box::new(Sleep),
            BehaviorState::InfluencedWork => Box::new(InfluencedWork),
        };
        let (new_velocity, new_state) = behavior.execute(entity, &position, hunger, &food_query, &time);
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
    for (mut hunger) in query.iter_mut() {
        hunger.0 += 3.0 * time.delta_seconds();
        hunger.0 = hunger.0.clamp(0.0, 100.0);
        // if hunger.0 > 30.0 {
        // info!("Prey {:?} hungry {:?}",prey, hunger.0);
        // }
    }
}

pub fn eat_food(
    mut commands: Commands,
    mut prey_query: Query<(&Position, &mut Hunger, &mut BehaviorState, Entity, &EntityType)>,
    food_query: Query<(Entity, &Position, &EntityType), (With<EntityType>, Without<BehaviorState>)>,
) {
    for (prey_pos, mut hunger, mut behavior_state, prey_entity, prey_type) in prey_query.iter_mut() {
        if *behavior_state == BehaviorState::SeekFood && matches!(prey_type, EntityType::Prey) {
            for (food_entity, food_pos, food_type) in food_query.iter() {
                if matches!(food_type, EntityType::Food) {
                    let distance = prey_pos.0.distance(food_pos.0);
                    if distance < 1.0 { // Eat only when very close
                        hunger.0 = (hunger.0 - 50.0).max(0.0);
                        commands.entity(food_entity).despawn();
                        *behavior_state = BehaviorState::SeekFood; // Re-seek food
                        info!(
                            "Prey {:?} (type: {:?}) ate Food {:?} (type: {:?}), hunger now {}, distance: {}",
                            prey_entity, prey_type, food_entity, food_type, hunger.0, distance
                        );
                    }
                }
            }
        }
    }
}

pub fn starve(
    mut commands: Commands,
    prey_query: Query<(Entity, &Hunger, &BehaviorState), With<EntityType>>,
    food_query: Query<&EntityType, With<EntityType>>,
) {
    let food_available = food_query.iter().any(|et| matches!(et, EntityType::Food));
    for (prey_entity, hunger, behavior_state) in prey_query.iter() {
        if hunger.0 >= 100.0 && !food_available && *behavior_state == BehaviorState::SeekFood {
            commands.entity(prey_entity).despawn();
            info!("Prey {:?} starved and died", prey_entity);
        }
    }
}