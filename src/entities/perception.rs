use bevy::prelude::*;
use super::components::{
    SpatialGrid, Genes, Perception, BehaviorState, Prey, Food, Predator, 
    WorldObject,
    SimulationSpeed,
};

const NEIGHBOR_CELLS:[IVec2;9] = [
    IVec2::new(-1, -1), IVec2::new(-1, 0), IVec2::new(-1, 1),
    IVec2::new(0, -1),  IVec2::new(0, 0),  IVec2::new(0, 1),
    IVec2::new(1, -1),  IVec2::new(1, 0),  IVec2::new(1, 1),
];
const NEARBY_AVOIDANCE_DISTANCE: f32 = 5.0;

pub fn perception_scan_system(
  grid: Res<SpatialGrid>,
  mut query: Query<(Entity, &Transform, &Genes, &mut Perception, &BehaviorState), With<Prey>>,
  lookup_query: Query<&Transform, With<WorldObject>>,
  food_query: Query<Entity, With<Food>>,
  predator_query: Query<Entity, With<Predator>>,
  time: Res<Time>,
  simulation_speed: Res<SimulationSpeed>
) {
  let delta_time = time.delta_seconds() * simulation_speed.0;
  for (
      entity, transform, genes, mut perception, behavior_state
  ) in query.iter_mut() {
      perception.time_since_last_sense += delta_time;
      perception.time_since_last_target += delta_time;
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