use bevy::prelude::*;
use rand::Rng;
use crate::helpers::util::{WORLD_WIDTH, WORLD_HEIGHT};
use super::components::{
    Genes, SpatialGrid, WorldObject,
    create_food, create_prey,
};

pub fn setup_entities(mut commands: Commands) {
    // Only spawn default entities if no config was loaded
    let mut rng = rand::thread_rng();
    // let mut positions: Vec<Vec2> = Vec::new();
    // let min_distance = 5.0;

    // // Helper: Check if a position is too close to existing ones
    // let is_position_valid = |pos: Vec2| {
    //     !positions.iter().any(|&existing| existing.distance(pos) < min_distance)
    // };

    // // Helper: Generate random position within world bounds
    // let mut random_pos = || Vec2::new(
    //     rng.gen_range(min_distance..WORLD_WIDTH - min_distance),
    //     rng.gen_range(min_distance..WORLD_HEIGHT - min_distance),
    // );

    for _ in 0..1000 {
        let pos = Vec2::new(
            rng.gen_range(0.0..WORLD_WIDTH),
            rng.gen_range(0.0..WORLD_HEIGHT),
        );
        // let mut pos = random_pos();
        // let mut attempts = 0;
        // while !is_position_valid(pos) && attempts < 100 {
        //     pos = random_pos();
        //     attempts += 1;
        // }
        // if attempts >= 100 {
        //     warn!("Could not find valid position for Food after 100 attempts. Using last position.");
        //     continue;
        // }
        // positions.push(pos);
        commands.spawn(create_prey(
            pos,
            rng.gen_range(0.0..30.0),
            Genes {
                vision_range: rng.gen_range(70.0..100.0),
                laziness: rng.gen_range(0.0..1.0),
                greed: rng.gen_range(0.0..1.0),
                curiosity: rng.gen_range(0.0..1.0),
                wander_radius: rng.gen_range(200.0..500.0),
                bite_size: rng.gen_range(1.0..10.0),
                max_speed: rng.gen_range(5.0..10.0),
                hunger_rate: rng.gen_range(0.5..1.0),
                // aggression: 0.0,
                // boldness: 0.0,
                // panic_threshold: 0.0,
                // smell_range: 0.0,
            }
        ));
    }
    for _ in 0..200 {
        commands.spawn(create_food(
            Vec2::new(
                rng.gen_range(0.0..WORLD_WIDTH),
                rng.gen_range(0.0..WORLD_HEIGHT),
            ),
            rng.gen_range(10.0..100.0)
        ));
    }
    info!("Spawned 1000 Prey and 200 Food entities");
}

pub fn update_grid_system(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Transform), With<WorldObject>>,
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
