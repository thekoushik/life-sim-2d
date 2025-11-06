use bevy::prelude::*;
use super::components::{
    Position, Hunger, BehaviorState, Genes, Perception, Prey, Food, FoodAmount, create_corpse, SimulationSpeed, Age
};

const COLLISION_RADIUS: f32 = 4.0;
const REPULSION_STRENGTH: f32 = 50.0;

pub fn game_loop(
  mut commands: Commands,
  mut prey_query: Query<(
      Entity,&mut Position, &mut Hunger, &mut BehaviorState, &Genes,
      &mut Transform,
      &Perception,
      &mut Age
  ), With<Prey>>,
  mut food_query: Query<(Entity,&Position, &mut FoodAmount), (With<Food>, Without<BehaviorState>)>,
  time: Res<Time>,
  simulation_speed: Res<SimulationSpeed>
) {
    let mut foods_to_delete = Vec::new();
    for (
        prey_entity,
        mut prey_pos,
        mut hunger,
        mut behavior_state,
        genes,
        mut transform,
        perception,
        mut age
    ) in prey_query.iter_mut() {
        let delta_time = time.delta_seconds() * simulation_speed.0;
        hunger.0 += genes.hunger_rate * delta_time;
        hunger.0 = hunger.0.clamp(0.0, 100.0);

        let mut nearest_food_pos = None;
        if let Some(food) = perception.target_food {
            if !foods_to_delete.contains(&food) {
                if let Ok((food_entity, food_pos, mut food_amount)) = food_query.get_mut(food) {
                    let distance = prey_pos.0.distance(food_pos.0);
                    if distance <= 2.5 {
                        let amount_eaten = genes.bite_size.min(food_amount.0);
                        food_amount.0 -= amount_eaten;
                        hunger.0 = (hunger.0 - amount_eaten).clamp(0.0, 100.0);
                        if food_amount.0 <= 0.0 {
                            // do not delete the food entity here, just add it to the list of foods to delete
                            // so others don't try to delete it again
                            foods_to_delete.push(food_entity);
                        }
                    } else {
                        nearest_food_pos = Some(food_pos.0);
                    }
                }
            }
        }
        let mut desired_velocity = Vec2::ZERO;
        if hunger.0 >= 100.0 {
            commands.entity(prey_entity).despawn();
            // TODO: implement corpse creation here and body flesh amount to be used for food amount
            commands.spawn(create_corpse(prey_pos.0));
        } else if let Some(food_pos) = nearest_food_pos {
            // Move toward nearest food
            let direction = (food_pos - prey_pos.0).normalize_or_zero();
            // more hungry = more speed
            let move_distance = (genes.max_speed * hunger.0 / 100.0) * delta_time; // Move at 10 units/s
            desired_velocity = direction * move_distance;
        } else if let Some(target) = perception.target {
            let direction = (target - prey_pos.0).normalize_or_zero();
            let move_distance = genes.max_speed * delta_time; // Move at wander speed
            desired_velocity = direction * move_distance;
        }

        // always avoid neighbors, cascade movement
        let mut avoidance_force = Vec2::ZERO;
        for &neighbor_pos in perception.neighbors.iter() {
            // its the food I am going to eat, so I should not avoid it
            if let Some(food_pos) = nearest_food_pos {
                if neighbor_pos == food_pos {
                    continue;
                }
            }
            let distance = prey_pos.0.distance(neighbor_pos);
            let repulsion_direction = (prey_pos.0 - neighbor_pos).normalize_or_zero();
            let strength = (COLLISION_RADIUS - distance) / COLLISION_RADIUS; // Stronger when closer
            avoidance_force += repulsion_direction * strength * REPULSION_STRENGTH * delta_time;


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
            let tangent_direction = if dot_product >= 0.0 { tangent } else { -tangent };
            // accumulate the tangent force for each neighbor
            avoidance_force += tangent_direction * delta_time;
        }

        prey_pos.0 += desired_velocity + avoidance_force;
        transform.translation = prey_pos.0.extend(0.0);
        if hunger.0 > 50.0 || (hunger.0 < 80.0 && genes.greed > 0.5) {
            *behavior_state = BehaviorState::SeekFood; // Re-seek new Food
        } else if genes.laziness > 0.5 {
            *behavior_state = BehaviorState::Sleep;
        } else {
            *behavior_state = BehaviorState::Wander;
        }

        age.0 += delta_time;
        if age.0 >= genes.max_age {
            commands.entity(prey_entity).despawn();
            commands.spawn(create_corpse(prey_pos.0));
        }
    }
    // Delete foods that are no longer needed
    for food_entity in foods_to_delete.iter_mut() {
        commands.entity(*food_entity).despawn();
    }
}