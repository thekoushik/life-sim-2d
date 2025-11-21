use super::components::{
    Corpse, CorpseState, Food, FoodAmount, Genes, LivingEntity, Needs, Perception, Position,
    Predator, Prey, SimulationSpeed, create_corpse,
};
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

const COLLISION_RADIUS: f32 = 4.0;
const COLLISION_REPULSION_STRENGTH: f32 = 50.0;

const CORPSE_FEAR_RADIUS: f32 = 10.0;
// const CORPSE_FEAR_REPULSION_STRENGTH: f32 = 100.0;
const FEAR_RADIUS: f32 = 20.0;

const MATE_ATTRACTION_RADIUS: f32 = 100.0;
const MATE_ATTRACTION_STRENGTH: f32 = 10.0;

pub fn game_loop_prey(
    mut commands: Commands,
    mut prey_query: Query<(&mut Position, &mut Needs, &Genes, &Perception), With<Prey>>,
    mut food_query: Query<(Entity, &Transform, &mut FoodAmount), With<Food>>,
    // lookup_gene_query: Query<&Genes, With<LivingEntity>>, //conflict
    // lookup_pos_query: Query<&Position, With<LivingEntity>>, //conflict
    time: Res<Time>,
    simulation_speed: Res<SimulationSpeed>,
) {
    let mut foods_to_delete = Vec::new();
    // let mut rng = rand::thread_rng();

    for (mut prey_pos, mut needs, genes, perception) in prey_query.iter_mut() {
        let delta_time = time.delta_seconds() * simulation_speed.0;

        let mut nearest_food_pos = None;
        if let Some(food) = perception.target_food {
            if !foods_to_delete.contains(&food) {
                if let Ok((food_entity, food_transform, mut food_amount)) = food_query.get_mut(food)
                {
                    let food_pos = food_transform.translation.truncate();
                    let distance = prey_pos.0.distance(food_pos);
                    if distance <= 2.5 {
                        let amount_eaten = genes.bite_size.min(food_amount.0);
                        food_amount.0 -= amount_eaten;
                        needs.hunger = (needs.hunger - amount_eaten).clamp(0.0, 100.0);
                        needs.energy += amount_eaten / 100.0; // eating food = energy gain
                        if food_amount.0 <= 0.0 {
                            // do not delete the food entity here, just add it to the list of foods to delete
                            // so others don't try to delete it again
                            foods_to_delete.push(food_entity);
                        }
                    } else {
                        nearest_food_pos = Some(food_pos);
                    }
                }
            }
        }
        let mut speed = genes.max_speed * needs.energy;
        if needs.sanity < 0.3 && rand::random::<f32>() > 0.5 {
            speed *= 1.0 - needs.sanity;
        }
        let mut desired_velocity = Vec2::ZERO;
        // TODO: hunger should also influence sanity, and energy
        if let Some(food_pos) = nearest_food_pos {
            // Move toward nearest food
            let direction = (food_pos - prey_pos.0).normalize_or_zero();
            // more hungry = more speed
            let move_distance = (genes.max_speed * needs.hunger / 100.0) * delta_time;
            desired_velocity = direction * move_distance;
        } else if let Some(target) = perception.target_position {
            let direction = (target - prey_pos.0).normalize_or_zero();
            let move_distance = speed * delta_time; // Move at wander speed
            desired_velocity = direction * move_distance;
        }

        // always avoid neighbors, cascade movement
        let mut avoidance_force = Vec2::ZERO;
        for &(neighbor_pos, strength) in perception.repulsions.iter() {
            // its the food I am going to eat, so I should not avoid it
            if let Some(food_pos) = nearest_food_pos {
                if neighbor_pos == food_pos {
                    continue;
                }
            }
            let distance = prey_pos.0.distance(neighbor_pos);
            let repulsion_direction = (prey_pos.0 - neighbor_pos).normalize_or_zero();
            // let strength = (COLLISION_RADIUS - distance) / COLLISION_RADIUS; // Stronger when closer
            avoidance_force +=
                repulsion_direction * strength * COLLISION_REPULSION_STRENGTH * delta_time;

            // Add tangential force for going around
            // randomly choose the tangent direction
            let tangent = if rand::random::<f32>() > 0.5 {
                Vec2::new(-repulsion_direction.y, repulsion_direction.x)
            } else {
                Vec2::new(repulsion_direction.y, -repulsion_direction.x)
            };

            // Choose tangent direction based on desired movement
            let desired_movement_direction = desired_velocity.normalize_or_zero();
            let dot_product = tangent.dot(desired_movement_direction);
            let tangent_direction = if dot_product >= 0.0 {
                tangent
            } else {
                -tangent
            };
            // accumulate the tangent force for each neighbor
            avoidance_force += tangent_direction * delta_time;
        }
        // check if there is a nearby mate
        // if genes.gender == false {
        //     let nearby_mates_count = perception.nearby_mates.len();
        //     // for females only
        //     for &mate_entity in perception.nearby_mates.iter() {
        //         if let Ok(mate_genes) = lookup_gene_query.get(mate_entity) {
        //             if nearby_mates_count == 1 {
        //                 needs.mating_timer += delta_time + rng.gen_range(0.1..0.2);
        //             } else {
        //                 needs.mating_timer += delta_time / nearby_mates_count as f32;
        //             }
        //             if needs.mating_timer >= 1.0 {
        //                 needs.mating_timer = 0.0;
        //                 needs.partner = Some(mate_entity);
        //                 needs.partner_timer = rng.gen_range(10.0..30.0);
        //                 needs.pregnant = true;
        //                 needs.pregnancy_timer = rng.gen_range(10.0..30.0);
        //                 needs.partner_genes = Some(mate_genes.clone());
        //                 break;
        //             }
        //         }
        //     }
        // }
        // let mut mate_attraction_force = Vec2::ZERO;
        // if let Some(mate_partner) = needs.partner {
        //     if let Ok(mate_pos) = lookup_pos_query.get(mate_partner) {
        //         let distance = prey_pos.0.distance(mate_pos.0);
        //         let attraction_direction = (mate_pos.0 - prey_pos.0).normalize_or_zero();
        //         let strength = (MATE_ATTRACTION_RADIUS - distance) / MATE_ATTRACTION_RADIUS; // Stronger when closer
        //         mate_attraction_force +=
        //             attraction_direction * strength * MATE_ATTRACTION_STRENGTH * delta_time;
        //     }
        // }

        // avoid corpses
        // let mut corpse_avoidance_force = Vec2::ZERO;
        // for &(corpse_pos, stench) in perception.nearby_corpses.iter() {
        //     let distance = prey_pos.0.distance(corpse_pos);
        //     let stench_radius = CORPSE_FEAR_RADIUS + (CORPSE_FEAR_RADIUS * stench);
        //     if distance > stench_radius {
        //         continue;
        //     }
        //     needs.sanity -= stench * delta_time;
        //     let repulsion_direction = (prey_pos.0 - corpse_pos).normalize_or_zero();
        //     let strength = (stench_radius - distance) / stench_radius; // Stronger when closer
        //     corpse_avoidance_force +=
        //         repulsion_direction * strength * COLLISION_REPULSION_STRENGTH * delta_time;
        // }

        // avoid predators
        // let mut predator_avoidance_force = Vec2::ZERO;
        // let fear_count = perception.visible_predators.len();
        // for &(predator_pos, threshold) in perception.visible_predators.iter() {
        //     let strength = threshold / fear_count as f32;
        //     needs.sanity -= strength * delta_time;
        //     let repulsion_direction = (prey_pos.0 - predator_pos).normalize_or_zero();
        //     predator_avoidance_force +=
        //         repulsion_direction * strength * COLLISION_REPULSION_STRENGTH * delta_time;
        // }

        // TODO: unify all the forces into one vector and apply it to the prey
        prey_pos.0 += desired_velocity + avoidance_force; // + corpse_avoidance_force + predator_avoidance_force;
    }
    // Delete foods that are no longer needed
    for food_entity in foods_to_delete.iter_mut() {
        commands.entity(*food_entity).despawn();
    }
}

pub fn game_loop_predator(
    mut commands: Commands,
    mut predator_query: Query<(&mut Position, &mut Needs, &Genes, &Perception), With<Predator>>,
    mut prey_query: Query<(Entity, &Transform, &mut FoodAmount), With<Prey>>,
    mut corpse_query: Query<(&mut CorpseState, &Transform), With<Corpse>>,
    // lookup_gene_query: Query<&Genes, With<LivingEntity>>, //conflict
    // lookup_pos_query: Query<&Position, With<LivingEntity>>, //conflict
    time: Res<Time>,
    simulation_speed: Res<SimulationSpeed>,
) {
    let mut prey_to_die: HashMap<Entity, Vec2> = HashMap::new();
    let mut rng = rand::thread_rng();

    for (mut predator_pos, mut needs, genes, perception) in predator_query.iter_mut() {
        let delta_time = time.delta_seconds() * simulation_speed.0;

        let mut nearest_food_pos = None;
        if let Some(food) = perception.target_food {
            if !prey_to_die.contains_key(&food) {
                if let Ok((prey_entity, prey_transform, mut food_amount)) = prey_query.get_mut(food)
                {
                    let prey_pos = prey_transform.translation.truncate();
                    let distance = predator_pos.0.distance(prey_pos);
                    if distance <= COLLISION_RADIUS {
                        let amount_eaten = genes.bite_size.min(food_amount.0);
                        food_amount.0 -= amount_eaten;
                        needs.hunger = (needs.hunger - amount_eaten).clamp(0.0, 100.0);
                        needs.energy += amount_eaten / 100.0; // eating food = energy gain
                        if food_amount.0 <= 0.0 {
                            // do not delete the food entity here, just add it to the list of foods to delete
                            // so others don't try to delete it again
                            prey_to_die.insert(prey_entity, prey_pos);
                        }
                    } else {
                        nearest_food_pos = Some(prey_pos);
                    }
                }
                if let Ok((mut corpse_state, transform)) = corpse_query.get_mut(food) {
                    let corpse_pos = transform.translation.truncate();
                    let distance = predator_pos.0.distance(corpse_pos);
                    if distance <= 2.5 {
                        let amount_eaten = genes.bite_size.min(corpse_state.flesh_amount);
                        corpse_state.flesh_amount -= amount_eaten;
                        needs.hunger = (needs.hunger - amount_eaten).clamp(0.0, 100.0);
                        needs.energy += amount_eaten / 100.0; // eating food = energy gain
                    }
                }
            }
        }
        let mut speed = genes.max_speed * needs.energy;
        if needs.sanity < 0.3 && rand::random::<f32>() > 0.5 {
            speed *= 1.0 - needs.sanity;
        }
        let mut desired_velocity = Vec2::ZERO;
        // TODO: hunger should also influence sanity, and energy
        if let Some(food_pos) = nearest_food_pos {
            // Move toward nearest food
            let direction = (food_pos - predator_pos.0).normalize_or_zero();
            // more hungry = more speed
            let move_distance = (genes.max_speed * needs.hunger / 100.0) * delta_time;
            desired_velocity = direction * move_distance;
        } else if let Some(target) = perception.target_position {
            let direction = (target - predator_pos.0).normalize_or_zero();
            let move_distance = speed * delta_time; // Move at wander speed
            desired_velocity = direction * move_distance;
        }

        // always avoid neighbors, cascade movement
        let mut avoidance_force = Vec2::ZERO;
        for &(neighbor_pos, strength) in perception.repulsions.iter() {
            // its the food I am going to eat, so I should not avoid it
            if let Some(food_pos) = nearest_food_pos {
                if neighbor_pos == food_pos {
                    continue;
                }
            }
            // let distance = predator_pos.0.distance(neighbor_pos);
            let repulsion_direction = (predator_pos.0 - neighbor_pos).normalize_or_zero();
            // let strength = (COLLISION_RADIUS - distance) / COLLISION_RADIUS; // Stronger when closer
            avoidance_force +=
                repulsion_direction * strength * COLLISION_REPULSION_STRENGTH * delta_time;

            // Add tangential force for going around
            // randomly choose the tangent direction
            let tangent = if rand::random::<f32>() > 0.5 {
                Vec2::new(-repulsion_direction.y, repulsion_direction.x)
            } else {
                Vec2::new(repulsion_direction.y, -repulsion_direction.x)
            };

            // Choose tangent direction based on desired movement
            let desired_movement_direction = desired_velocity.normalize_or_zero();
            let dot_product = tangent.dot(desired_movement_direction);
            let tangent_direction = if dot_product >= 0.0 {
                tangent
            } else {
                -tangent
            };
            // accumulate the tangent force for each neighbor
            avoidance_force += tangent_direction * delta_time;
        }
        // check if there is a nearby mate
        // if genes.gender == false {
        //     let nearby_mates_count = perception.nearby_mates.len();
        //     // for females only
        //     for &mate_entity in perception.nearby_mates.iter() {
        //         if let Ok(mate_genes) = lookup_gene_query.get(mate_entity) {
        //             if nearby_mates_count == 1 {
        //                 needs.mating_timer += delta_time + rng.gen_range(0.1..0.2);
        //             } else {
        //                 needs.mating_timer += delta_time / nearby_mates_count as f32;
        //             }
        //             if needs.mating_timer >= 1.0 {
        //                 needs.mating_timer = 0.0;
        //                 needs.partner = Some(mate_entity);
        //                 needs.partner_timer = rng.gen_range(10.0..30.0);
        //                 needs.pregnant = true;
        //                 needs.pregnancy_timer = rng.gen_range(10.0..30.0);
        //                 needs.partner_genes = Some(mate_genes.clone());
        //                 break;
        //             }
        //         }
        //     }
        // }
        // let mut mate_attraction_force = Vec2::ZERO;
        // if let Some(mate_partner) = needs.partner {
        //     if let Ok(mate_pos) = lookup_pos_query.get(mate_partner) {
        //         let distance = prey_pos.0.distance(mate_pos.0);
        //         let attraction_direction = (mate_pos.0 - prey_pos.0).normalize_or_zero();
        //         let strength = (MATE_ATTRACTION_RADIUS - distance) / MATE_ATTRACTION_RADIUS; // Stronger when closer
        //         mate_attraction_force +=
        //             attraction_direction * strength * MATE_ATTRACTION_STRENGTH * delta_time;
        //     }
        // }

        // avoid corpses
        // let mut corpse_avoidance_force = Vec2::ZERO;
        // for &(corpse_pos, stench) in perception.nearby_corpses.iter() {
        //     let distance = prey_pos.0.distance(corpse_pos);
        //     let stench_radius = CORPSE_FEAR_RADIUS + (CORPSE_FEAR_RADIUS * stench);
        //     if distance > stench_radius {
        //         continue;
        //     }
        //     needs.sanity -= stench * delta_time;
        //     let repulsion_direction = (prey_pos.0 - corpse_pos).normalize_or_zero();
        //     let strength = (stench_radius - distance) / stench_radius; // Stronger when closer
        //     corpse_avoidance_force +=
        //         repulsion_direction * strength * COLLISION_REPULSION_STRENGTH * delta_time;
        // }

        // avoid predators
        // let mut predator_avoidance_force = Vec2::ZERO;
        // let fear_count = perception.visible_predators.len();
        // for &(predator_pos, threshold) in perception.visible_predators.iter() {
        //     let strength = threshold / fear_count as f32;
        //     needs.sanity -= strength * delta_time;
        //     let repulsion_direction = (prey_pos.0 - predator_pos).normalize_or_zero();
        //     predator_avoidance_force +=
        //         repulsion_direction * strength * COLLISION_REPULSION_STRENGTH * delta_time;
        // }

        // TODO: unify all the forces into one vector and apply it to the prey
        predator_pos.0 += desired_velocity + avoidance_force; // + corpse_avoidance_force + predator_avoidance_force;
    }
    // Delete foods that are no longer needed
    for (prey_entity, pos) in prey_to_die.iter_mut() {
        commands.entity(*prey_entity).despawn();
        // TODO: implement corpse creation here and body flesh amount to be used for food amount
        commands.spawn(create_corpse(*pos, rng.gen_range(10.0..50.0)));
    }
}
