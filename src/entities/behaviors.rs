use bevy::prelude::*;
use crate::entities::components::{Position, Hunger, BehaviorState, EntityType};
use rand::Rng;

pub trait Behavior {
    fn execute(
        &self,
        entity: Entity,
        position: &Position,
        hunger: &Hunger,
        food_query: &Query<(&Position, &EntityType), Without<BehaviorState>>,
        time: &Res<Time>,
    ) -> (Vec2, BehaviorState);
}

pub struct SeekFood;

impl Behavior for SeekFood {
    fn execute(
        &self,
        _entity: Entity,
        position: &Position,
        hunger: &Hunger,
        food_query: &Query<(&Position, &EntityType), Without<BehaviorState>>,
        time: &Res<Time>,
    ) -> (Vec2, BehaviorState) {
        // Find nearest food
        let mut nearest_food_pos = None;
        let mut min_distance = f32::MAX;
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
        let velocity = if let Some(food_pos) = nearest_food_pos {
            let direction = (food_pos - position.0).normalize_or_zero();
            direction * 5.0 * time.delta_seconds() // Move at 50 units/s
        } else {
            Vec2::ZERO // No food: stay still, will starve if hungry
        };

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

        (velocity, next_state)
    }
}

pub struct Sleep;

impl Behavior for Sleep {
    fn execute(
        &self,
        _entity: Entity,
        _position: &Position,
        hunger: &Hunger,
        _food_query: &Query<(&Position, &EntityType), Without<BehaviorState>>,
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
        _entity: Entity,
        _position: &Position,
        hunger: &Hunger,
        _food_query: &Query<(&Position, &EntityType), Without<BehaviorState>>,
        time: &Res<Time>,
    ) -> (Vec2, BehaviorState) {
        let mut rng = rand::thread_rng();
        let direction = Vec2::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        ).normalize_or_zero();
        let velocity = direction * 5.0 * time.delta_seconds(); // Move at 5 units/s

        let next_state = if hunger.0 > 50.0 {
            BehaviorState::SeekFood
        } else {
            BehaviorState::InfluencedWork
        };
        (velocity, next_state)
    }
}