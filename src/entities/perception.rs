use crate::entities::components::LivingEntity;

use super::components::{
    BehaviorState, Corpse, CorpseState, Food, Genes, Needs, Perception, Position, Predator, Prey,
    SimulationSpeed, SpatialGrid, SpeciesId, WorldObject,
};
use bevy::prelude::*;

const NEIGHBOR_CELLS: [IVec2; 9] = [
    IVec2::new(-1, -1),
    IVec2::new(-1, 0),
    IVec2::new(-1, 1),
    IVec2::new(0, -1),
    IVec2::new(0, 0),
    IVec2::new(0, 1),
    IVec2::new(1, -1),
    IVec2::new(1, 0),
    IVec2::new(1, 1),
];
const NEARBY_AVOIDANCE_DISTANCE: f32 = 5.0;
const MATE_DETECTION_DISTANCE: f32 = 10.0;
const FEAR_RADIUS: f32 = 20.0;
const COLLISION_RADIUS: f32 = 4.0;

pub fn perception_scan_system_prey(
    grid: Res<SpatialGrid>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &Genes,
            &mut Perception,
            &BehaviorState,
            &Needs,
            &SpeciesId,
        ),
        With<Prey>,
    >,
    lookup_query: Query<&Position, With<WorldObject>>,
    food_query: Query<Entity, With<Food>>,
    predator_query: Query<Entity, With<Predator>>,
    corpse_query: Query<&CorpseState, With<Corpse>>,
    needs_query: Query<(&Needs, &Genes, &SpeciesId), With<LivingEntity>>,
    time: Res<Time>,
    simulation_speed: Res<SimulationSpeed>,
) {
    let delta_time = time.delta_seconds() * simulation_speed.0;
    for (entity, transform, genes, mut perception, behavior_state, needs, species_id) in
        query.iter_mut()
    {
        perception.time_since_last_sense += delta_time;
        let mut skip_sense = false;
        let laziness_threshold = genes.laziness * 5.0 * simulation_speed.0;
        let fear_threshold = genes.boldness * 2.0 * simulation_speed.0;

        if perception.time_since_last_sense < laziness_threshold
            || *behavior_state == BehaviorState::Sleep
        {
            // too lazy or sleeping would not be able to see nearby entities
            // but should know the position of nearby entities so we can avoid them
            skip_sense = true;
        } else if perception.time_since_last_sense >= laziness_threshold {
            perception.time_since_last_sense = 0.0;
        }
        // we update neighbors always even if they are lazy or sleeping
        // so we have advantage for other systems to know the position of nearby entities
        perception.repulsions.clear();
        perception.attractions.clear();
        if !skip_sense {
            perception.target_food = None;

            if *behavior_state == BehaviorState::Wander || needs.sanity < 0.1 {
                // curiosity determines how often the target changes when wandering
                let change_interval = 3.0.lerp(12.0, 1.0 - genes.curiosity);
                if perception.time_since_last_target > change_interval {
                    perception.time_since_last_target = 0.0;
                    let angle = rand::random::<f32>() * std::f32::consts::TAU;
                    let distance = if needs.sanity < 0.1 {
                        genes.wander_radius
                    } else {
                        rand::random::<f32>() * genes.wander_radius
                    };
                    perception.target_position =
                        Some(transform.translation.truncate() + Vec2::from_angle(angle) * distance);
                }
            } else {
                perception.target_position = None;
            }
        }

        let pos = transform.translation.truncate();
        let cell = IVec2::new(
            (pos.x / grid.cell_size).floor() as i32,
            (pos.y / grid.cell_size).floor() as i32,
        );
        let mut visible_food: Vec<(Entity, f32)> = Vec::new();
        let mut closest_food_dist: f32 = f32::INFINITY;
        let mut closest_food_entity: Option<Entity> = None;
        for offset in NEIGHBOR_CELLS {
            if let Some(entities) = grid.buckets.get(&(cell + offset)) {
                for &other in entities {
                    if other == entity {
                        continue;
                    }
                    let Ok(other_pos) = lookup_query.get(other) else {
                        continue;
                    };

                    let dist = pos.distance(other_pos.0);

                    if dist < NEARBY_AVOIDANCE_DISTANCE {
                        // very close position occupied by something
                        perception
                            .repulsions
                            .push((other_pos.0, (COLLISION_RADIUS - dist) / COLLISION_RADIUS)); //(COLLISION_RADIUS - distance) / COLLISION_RADIUS
                    }
                    if dist < MATE_DETECTION_DISTANCE && needs.mate_ready {
                        if let Ok((other_needs, other_genes, other_species_id)) =
                            needs_query.get(other)
                        {
                            // always choose same species for mating
                            if other_needs.mate_ready
                                && other_genes.gender != genes.gender
                                && other_species_id.0 == species_id.0
                            {
                                perception.attractions.push((other_pos.0, 1.0));
                            }
                        }
                    }
                    if !skip_sense {
                        // if other is corpse
                        if let Ok(corpse_state) = corpse_query.get(other) {
                            if dist < genes.vision_range {
                                // stench is stronger when closer and weaker when further away
                                let stench = (100.0 - corpse_state.decay_timer) / dist;
                                perception.repulsions.push((other_pos.0, stench));
                            }
                        }
                        if needs.sanity > 0.1 {
                            // if other is food
                            if food_query.get(other).is_ok() && dist < genes.vision_range {
                                visible_food.push((other, dist));
                                if dist < closest_food_dist {
                                    closest_food_dist = dist;
                                    closest_food_entity = Some(other);
                                }
                            }
                        }
                    }
                    if predator_query.get(other).is_ok() && dist <= FEAR_RADIUS {
                        perception
                            .repulsions
                            .push((other_pos.0, dist / FEAR_RADIUS));
                    }
                }
            }
        }
        if !visible_food.is_empty() {
            if rand::random::<f32>() < 0.5 {
                // 50% chance to prefer closer target
                perception.target_food = closest_food_entity;
            } else {
                // 50% chance to make a "mistake" and pick a random one
                let idx = rand::random::<usize>() % visible_food.len();
                perception.target_food = Some(visible_food[idx].0);
            }
        }
        // info!("Entity {:?} sees {} food", entity, perception.visible_food.len() );
    }
}

pub fn perception_scan_system_predator(
    grid: Res<SpatialGrid>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &Genes,
            &mut Perception,
            &BehaviorState,
            &Needs,
            &SpeciesId,
        ),
        With<Predator>,
    >,
    lookup_query: Query<&Position, With<WorldObject>>,
    prey_query: Query<Entity, With<Prey>>,
    corpse_query: Query<&CorpseState, With<Corpse>>,
    needs_query: Query<(&Needs, &Genes, &SpeciesId), With<LivingEntity>>,
    time: Res<Time>,
    simulation_speed: Res<SimulationSpeed>,
) {
    let delta_time = time.delta_seconds() * simulation_speed.0;
    for (entity, transform, genes, mut perception, behavior_state, needs, species_id) in
        query.iter_mut()
    {
        perception.time_since_last_sense += delta_time;
        let mut skip_sense = false;
        let laziness_threshold = genes.laziness * 5.0 * simulation_speed.0;
        // let fear_threshold = genes.boldness * 2.0 * simulation_speed.0;

        if perception.time_since_last_sense < laziness_threshold
            || *behavior_state == BehaviorState::Sleep
        {
            // too lazy or sleeping would not be able to see nearby entities
            // but should know the position of nearby entities so we can avoid them
            skip_sense = true;
        } else if perception.time_since_last_sense >= laziness_threshold {
            perception.time_since_last_sense = 0.0;
        }
        // we update neighbors always even if they are lazy or sleeping
        // so we have advantage for other systems to know the position of nearby entities
        perception.repulsions.clear();
        perception.attractions.clear();
        if !skip_sense {
            perception.target_food = None;

            if *behavior_state == BehaviorState::Wander || needs.sanity < 0.1 {
                // curiosity determines how often the target changes when wandering
                let change_interval = 3.0.lerp(12.0, 1.0 - genes.curiosity);
                if perception.time_since_last_target > change_interval {
                    perception.time_since_last_target = 0.0;
                    let angle = rand::random::<f32>() * std::f32::consts::TAU;
                    let distance = if needs.sanity < 0.1 {
                        genes.wander_radius
                    } else {
                        rand::random::<f32>() * genes.wander_radius
                    };
                    perception.target_position =
                        Some(transform.translation.truncate() + Vec2::from_angle(angle) * distance);
                }
            } else {
                perception.target_position = None;
            }
        }

        let pos = transform.translation.truncate();
        let cell = IVec2::new(
            (pos.x / grid.cell_size).floor() as i32,
            (pos.y / grid.cell_size).floor() as i32,
        );
        let mut visible_food: Vec<(Entity, f32)> = Vec::new();
        let mut closest_food_dist: f32 = f32::INFINITY;
        let mut closest_food_entity: Option<Entity> = None;
        for offset in NEIGHBOR_CELLS {
            if let Some(entities) = grid.buckets.get(&(cell + offset)) {
                for &other in entities {
                    if other == entity {
                        continue;
                    }
                    let Ok(other_pos) = lookup_query.get(other) else {
                        continue;
                    };

                    let dist = pos.distance(other_pos.0);

                    if dist < NEARBY_AVOIDANCE_DISTANCE {
                        // very close position occupied by something
                        perception
                            .repulsions
                            .push((other_pos.0, (COLLISION_RADIUS - dist) / COLLISION_RADIUS)); //(COLLISION_RADIUS - distance) / COLLISION_RADIUS
                    }
                    if dist < MATE_DETECTION_DISTANCE && needs.mate_ready {
                        if let Ok((other_needs, other_genes, other_species_id)) =
                            needs_query.get(other)
                        {
                            // always choose same species for mating
                            if other_needs.mate_ready
                                && other_genes.gender != genes.gender
                                && other_species_id.0 == species_id.0
                            {
                                perception.attractions.push((other_pos.0, 1.0));
                            }
                        }
                    }
                    if !skip_sense && needs.sanity > 0.1 {
                        // if other is corpse
                        if let Ok(corpse_state) = corpse_query.get(other) {
                            // fresh corpse is a food source
                            if dist < genes.vision_range
                                && corpse_state.flesh_amount > 0.0
                                && corpse_state.decay_timer > 75.0
                            {
                                //potential food source
                                visible_food.push((other, dist));
                                if dist < closest_food_dist {
                                    closest_food_dist = dist;
                                    closest_food_entity = Some(other);
                                }
                            }
                        }
                        // if other is food
                        if prey_query.get(other).is_ok() && dist < genes.vision_range {
                            visible_food.push((other, dist));
                            if dist < closest_food_dist {
                                closest_food_dist = dist;
                                closest_food_entity = Some(other);
                            }
                        }
                    }
                    // if predator_query.get(other).is_ok() && dist <= FEAR_RADIUS {
                    //     perception
                    //         .repulsions
                    //         .push((other_pos.0, dist / FEAR_RADIUS));
                    // }
                }
            }
        }
        if !visible_food.is_empty() {
            if rand::random::<f32>() < 0.5 {
                // 50% chance to prefer closer target
                perception.target_food = closest_food_entity;
            } else {
                // 50% chance to make a "mistake" and pick a random one
                let idx = rand::random::<usize>() % visible_food.len();
                perception.target_food = Some(visible_food[idx].0);
            }
        }
        // info!("Entity {:?} sees {} food", entity, perception.visible_food.len() );
    }
}
