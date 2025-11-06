use bevy::prelude::*;
use rand::Rng;
use crate::helpers::util::{WORLD_WIDTH, WORLD_HEIGHT};
use super::components::{
    Genes, SpatialGrid, WorldObject,
    create_food, create_prey,
};
use noisy_bevy::simplex_noise_2d;

fn spawn_forest(commands: &mut Commands, forest_count: i32, size: f32) {
    let mut rng = rand::thread_rng();
    // first, choose n random areas
    // then we spawn food in those areas based on noise value
    // this will give us forest like areas
    let mut areas = Vec::new();
    for _ in 0..forest_count {
        areas.push(Vec2::new(
            rng.gen_range(0.0..WORLD_WIDTH),
            rng.gen_range(0.0..WORLD_HEIGHT),
        ));
    }
    let half_size = size / 2.0;
    for area in areas {
        let density = rng.gen_range(0.5..1.0);
        let food_count_in_area = (simplex_noise_2d(area) * half_size) + half_size;// might move this count to the area
        let offset = density * half_size;
        // in every area, spread food randomly
        for _ in 0..food_count_in_area as i32 {
            let pos = Vec2::new(
                rng.gen_range(area.x - offset..area.x + offset),
                rng.gen_range(area.y - offset..area.y + offset),
            );
            commands.spawn(create_food(pos, rng.gen_range(10.0..100.0)));
        }
    }
}

pub fn setup_entities(mut commands: Commands) {
    // Only spawn default entities if no config was loaded
    let mut rng = rand::thread_rng();
    // use noise to spawn food in a more natural way

    // spawn food randomly
    // for _ in 0..400 {
    //     commands.spawn(create_food(
    //         Vec2::new(
    //             rng.gen_range(0.0..WORLD_WIDTH),
    //             rng.gen_range(0.0..WORLD_HEIGHT),
    //         ),
    //         rng.gen_range(10.0..100.0)
    //     ));
    // }
    // spawn area based food
    spawn_forest(&mut commands, rng.gen_range(5..15), rng.gen_range(100.0..200.0));

    for _ in 0..1000 {
        let pos = Vec2::new(
            rng.gen_range(0.0..WORLD_WIDTH),
            rng.gen_range(0.0..WORLD_HEIGHT),
        );
        commands.spawn(create_prey(
            pos,
            rng.gen_range(0.0..30.0),
            Genes {
                vision_range: rng.gen_range(300.0..500.0),
                laziness: rng.gen_range(0.0..1.0),
                greed: rng.gen_range(0.0..1.0),
                curiosity: rng.gen_range(0.0..1.0),
                wander_radius: rng.gen_range(300.0..600.0),
                bite_size: rng.gen_range(1.0..10.0),
                max_speed: rng.gen_range(5.0..10.0),
                hunger_rate: rng.gen_range(0.5..1.0),
                max_age: rng.gen_range(100.0..300.0),
                // aggression: 0.0,
                // boldness: 0.0,
                // panic_threshold: 0.0,
                // smell_range: 0.0,
            }
        ));
    }
    
    info!("Spawned foods and prey entities");
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
