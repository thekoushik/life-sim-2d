use bevy::prelude::*;
use crate::entities::components::{Position, Hunger, BehaviorState, EntityType, SenseRadius};
use rand::Rng;

pub trait Behavior {
    fn execute(
        &self,
        entity: Entity,
        position: &Position,
        hunger: &Hunger,
        sense_radius: &SenseRadius,
        food_query: &Query<(&Position, &EntityType), Without<BehaviorState>>,
        all_entities: &Query<(Entity, &Position, &EntityType), (With<EntityType>, Without<BehaviorState>)>,
        time: &Res<Time>,
    ) -> (Vec2, BehaviorState);
}

pub struct SeekFood;

impl Behavior for SeekFood {
    fn execute(
        &self,
        entity: Entity,
        position: &Position,
        hunger: &Hunger,
        sense_radius: &SenseRadius,
        food_query: &Query<(&Position, &EntityType), Without<BehaviorState>>,
        all_entities: &Query<(Entity, &Position, &EntityType), (With<EntityType>, Without<BehaviorState>)>,
        time: &Res<Time>,
    ) -> (Vec2, BehaviorState) {
        // Find nearest food
        let mut nearest_food_pos = None;
        let mut min_distance = sense_radius.0;
        for (food_pos, entity_type) in food_query.iter() {
            if matches!(entity_type, EntityType::Food) {
                let distance = position.0.distance(food_pos.0);
                if distance < min_distance {
                    min_distance = distance;
                    nearest_food_pos = Some(food_pos.0);
                }
            }
        }

        // Move toward nearest food or stay still if none found
        let mut desired_velocity = if let Some(food_pos) = nearest_food_pos {
            let direction = (food_pos - position.0).normalize_or_zero();
            // more hungry = more speed
            direction * (10.0 * hunger.0 / 100.0) * time.delta_seconds() // Move at 10 units/s
        } else {
            Vec2::ZERO // No food: stay still, will starve if hungry
        };

        // Apply collision avoidance (repulsive force from nearby entities)
        let mut avoidance_force = Vec2::ZERO;
        for (other_entity, other_pos, other_type) in all_entities.iter() {
            if other_entity == entity {
                continue; // Skip self
            }
            let distance = position.0.distance(other_pos.0);
            let collision_radius = match other_type {
                EntityType::Prey => 3.0, // Prey radius
                EntityType::Food => 1.0, // Food radius
            };
            if distance < collision_radius * 2.0 && distance > 0.0 {
                let direction = (position.0 - other_pos.0).normalize_or_zero();
                let strength = (collision_radius * 2.0 - distance) / (collision_radius * 2.0); // Stronger when closer
                avoidance_force += direction * strength * 50.0 * time.delta_seconds();
            }
        }

        // Combine desired velocity and avoidance force
        desired_velocity += avoidance_force;
        if desired_velocity.length() > 50.0 * time.delta_seconds() {
            desired_velocity = desired_velocity.normalize_or_zero() * 50.0 * time.delta_seconds();
        }

        // Transition to Sleep or InfluencedWork if not hungry
        let next_state = if hunger.0 <= 50.0 {
            if rand::random::<f32>() < 0.5 {
                BehaviorState::Sleep
            } else {
                BehaviorState::InfluencedWork
            }
        } else {
            BehaviorState::SeekFood
        };

        (desired_velocity, next_state)
    }
}

pub struct Sleep;

impl Behavior for Sleep {
    fn execute(
        &self,
        _entity: Entity,
        _position: &Position,
        hunger: &Hunger,
        _sense_radius: &SenseRadius,
        _food_query: &Query<(&Position, &EntityType), Without<BehaviorState>>,
        _all_entities: &Query<(Entity, &Position, &EntityType), (With<EntityType>, Without<BehaviorState>)>,
        _time: &Res<Time>,
    ) -> (Vec2, BehaviorState) {
        let next_state = if hunger.0 > 50.0 {
            BehaviorState::SeekFood
        } else {
            BehaviorState::Sleep
        };
        (Vec2::ZERO, next_state)
    }
}

pub struct InfluencedWork;

impl Behavior for InfluencedWork {
    fn execute(
        &self,
        entity: Entity,
        position: &Position,
        hunger: &Hunger,
        _sense_radius: &SenseRadius,
        _food_query: &Query<(&Position, &EntityType), Without<BehaviorState>>,
        all_entities: &Query<(Entity, &Position, &EntityType), (With<EntityType>, Without<BehaviorState>)>,
        time: &Res<Time>,
    ) -> (Vec2, BehaviorState) {
        // Slow random movement
        let mut rng = rand::thread_rng();
        let mut desired_velocity = Vec2::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        ).normalize_or_zero() * 5.0 * time.delta_seconds(); // Move at 5 units/s

        // Apply collision avoidance
        let mut avoidance_force = Vec2::ZERO;
        for (other_entity, other_pos, other_type) in all_entities.iter() {
            if other_entity == entity {
                continue;
            }
            let distance = position.0.distance(other_pos.0);
            let collision_radius = match other_type {
                EntityType::Prey => 3.0,
                EntityType::Food => 1.0,
            };
            if distance < collision_radius * 2.0 && distance > 0.0 {
                let direction = (position.0 - other_pos.0).normalize_or_zero();
                let strength = (collision_radius * 2.0 - distance) / (collision_radius * 2.0);
                avoidance_force += direction * strength * 5.0 * time.delta_seconds();
            }
        }

        // Combine desired velocity and avoidance force
        desired_velocity += avoidance_force;
        if desired_velocity.length() > 5.0 * time.delta_seconds() {
            desired_velocity = desired_velocity.normalize_or_zero() * 5.0 * time.delta_seconds();
        }

        let next_state = if hunger.0 > 50.0 {
            BehaviorState::SeekFood
        } else {
            BehaviorState::InfluencedWork
        };
        (desired_velocity, next_state)
    }
}