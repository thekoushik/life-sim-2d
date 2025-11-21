use super::components::{
    Age, BehaviorState, Corpse, CorpseState, Genes, LivingEntity, Needs, Position, Prey,
    SimulationSpeed, SpatialGrid, Species, SpeciesId, WorldObject, create_corpse, create_food,
    create_predator, create_prey,
};
use crate::helpers::util::{WORLD_HEIGHT, WORLD_WIDTH};
use bevy::{prelude::*, window::PrimaryWindow};
use noisy_bevy::simplex_noise_2d;
use rand::Rng;
const DEFAULT_SANITY_GAIN_RATE: f32 = 0.01;
const MATE_DETECTION_AGE_THRESHOLD_MIN: f32 = 0.2;
const MATE_DETECTION_AGE_THRESHOLD_MAX: f32 = 0.8;
const MATE_READY_SANITY_THRESHOLD: f32 = 0.5;
const MATE_READY_HUNGER_THRESHOLD: f32 = 90.0;
const MATE_READY_ENERGY_THRESHOLD: f32 = 0.9;

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
        let food_count_in_area = (simplex_noise_2d(area) * half_size) + half_size; // might move this count to the area
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
    // spawn area based food
    spawn_forest(
        &mut commands,
        rng.gen_range(20..30),
        rng.gen_range(100.0..200.0),
    );

    let mut vec_species = Vec::new();
    let species_count = rng.gen_range(5..10);
    for i in 0..species_count {
        let genetic_min = Genes::default();
        vec_species.push(Species {
            id: SpeciesId(i as u32),
            genetic_min: genetic_min.clone(),
            genetic_max: genetic_min.random_variation(),
        });
    }
    let predator_species_count = rng.gen_range(3..5);
    for i in 0..predator_species_count {
        let genetic_min = Genes::new_predator();
        vec_species.push(Species {
            id: SpeciesId(species_count + i as u32),
            genetic_min: genetic_min.clone(),
            genetic_max: genetic_min.random_variation(),
        });
    }

    for _ in 0..2000 {
        let pos = Vec2::new(
            rng.gen_range(0.0..WORLD_WIDTH),
            rng.gen_range(0.0..WORLD_HEIGHT),
        );
        let species = vec_species[rng.gen_range(0..vec_species.len())];
        commands.spawn(create_prey(pos, species.id, species.random_gene()));
    }
    for _ in 0..100 {
        let pos = Vec2::new(
            rng.gen_range(0.0..WORLD_WIDTH),
            rng.gen_range(0.0..WORLD_HEIGHT),
        );
        let species = vec_species[rng.gen_range(0..vec_species.len())];
        commands.spawn(create_predator(pos, species.id, species.random_gene()));
    }

    info!("Spawned foods, prey and predator entities");
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

pub fn update_entities(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Needs,
            &Genes,
            &mut Age,
            &Position,
            &mut BehaviorState,
            &mut Transform,
            &SpeciesId,
        ),
        With<LivingEntity>,
    >,
    mut corpse_query: Query<(Entity, &mut CorpseState, &Position), With<Corpse>>,
    // needs_query: Query<&Needs, With<LivingEntity>>,
    time: Res<Time>,
    simulation_speed: Res<SimulationSpeed>,
) {
    let delta_time = time.delta_seconds() * simulation_speed.0;
    let mut rng = rand::thread_rng();
    // update needs and age
    for (entity, mut needs, genes, mut age, pos, mut behavior_state, mut transform, species_id) in
        query.iter_mut()
    {
        let mut sanity_gain = DEFAULT_SANITY_GAIN_RATE;
        needs.hunger += genes.hunger_rate * delta_time;
        needs.hunger = needs.hunger.clamp(0.0, 100.0);
        if needs.hunger > 90.0 {
            sanity_gain = -0.1; // hungry = sanity decrease
        }
        if needs.hunger >= 100.0 {
            needs.energy *= 0.1 * delta_time; // max hungry = energy decrease
        } else {
            // TODO: verify this formula
            needs.energy += 0.1 * delta_time * ((100.0 - needs.hunger) / 100.0); // less hungry = more energy
        }
        needs.energy = needs.energy.clamp(0.0, 1.0);
        age.0 += delta_time;
        needs.sanity += delta_time * sanity_gain;
        needs.sanity = needs.sanity.clamp(0.0, 1.0);
        needs.mate_ready = needs.partner.is_none()
            && age.0 >= MATE_DETECTION_AGE_THRESHOLD_MIN
            && age.0 <= MATE_DETECTION_AGE_THRESHOLD_MAX
            && needs.sanity >= MATE_READY_SANITY_THRESHOLD
            && needs.hunger < MATE_READY_HUNGER_THRESHOLD
            && needs.energy >= MATE_READY_ENERGY_THRESHOLD
            && !needs.pregnant
            && needs.pregnancy_timer <= 0.0
            && rand::random::<f32>() < genes.sociality;
        // if there is a partner, decrease the partner timer
        if needs.partner.is_some() {
            needs.partner_timer -= delta_time;
            if needs.partner_timer <= 0.0 {
                needs.partner = None;
                needs.partner_timer = 0.0;
            }
        }
        needs.mating_timer -= delta_time;
        needs.mating_timer = needs.mating_timer.clamp(0.0, 1.0);
        if genes.gender == false {
            needs.pregnancy_timer -= delta_time;
            needs.pregnancy_timer = needs.pregnancy_timer.clamp(0.0, 1.0);
        }

        // update partner(for male)
        // if genes.gender == true && needs.partner.is_none() && needs.mate_ready {
        //     for &mate_entity in perception.nearby_mates.iter() {
        //         if let Ok(mate_needs) = needs_query.get(mate_entity) {
        //             if mate_needs.partner == Some(entity) {
        //                 // I am the partner of the other entity
        //                 needs.partner = Some(mate_entity);
        //                 needs.partner_timer = rng.gen_range(10.0..30.0);
        //                 break;
        //             }
        //         }
        //     }
        // }

        transform.translation = pos.0.extend(0.0);
        if needs.hunger > 50.0 || (needs.hunger < 80.0 && genes.greed > 0.5) {
            *behavior_state = BehaviorState::SeekFood; // Re-seek new Food
        } else if genes.laziness > 0.5 {
            *behavior_state = BehaviorState::Sleep;
        } else {
            *behavior_state = BehaviorState::Wander;
        }

        // update age and death
        if age.0 >= genes.max_age || (needs.hunger >= 100.0 && needs.energy <= 0.0) {
            commands.entity(entity).despawn();
            // TODO: implement corpse creation here and body flesh amount to be used for food amount
            commands.spawn(create_corpse(pos.0, rng.gen_range(10.0..50.0)));
        } else if needs.pregnant && needs.partner.is_some() {
            // update pregnancy
            if needs.pregnancy_timer <= 0.0 {
                needs.pregnant = false;
                needs.energy *= 0.1; // energy decrease after giving birth
                needs.pregnancy_timer = rng.gen_range(3.0..6.0); // backoff timer after giving birth
                // spawn offspring
                let offspring_count = if genes.max_offspring_count < 2 {
                    1
                } else {
                    rng.gen_range(1..genes.max_offspring_count)
                };
                let father_genes = needs.partner_genes.clone().unwrap();
                for _ in 0..offspring_count {
                    let new_genes = genes.mutate(&father_genes);
                    let mut child = create_prey(pos.0, species_id.clone(), new_genes);
                    child.10.mother = Some(entity); // set the mother of the child
                    commands.spawn(child);
                }
                needs.partner = None;
                needs.partner_genes = None;
                // needs.partner_timer = 0.0;
            }
        }
    }
    for (entity, mut corpse_state, pos) in corpse_query.iter_mut() {
        corpse_state.decay_timer -= delta_time * corpse_state.decay_rate;
        if corpse_state.decay_timer <= 0.0 {
            commands.entity(entity).despawn();
            commands.spawn(create_food(pos.0, corpse_state.flesh_amount));
        }
    }
}

fn mouse_to_world(
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec2> {
    let (camera, camera_transform) = q_camera.single();
    let window = q_windows.single();

    if let Some(screen_position) = window.cursor_position() {
        // Convert screen position to world position
        return camera
            .viewport_to_world(camera_transform, screen_position)
            .map(|ray| ray.origin.truncate()); // For 2D, the origin is on the Z=0 plane
    } else {
        None
    }
}

pub fn handle_input(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let mut rng = rand::thread_rng();
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(world_position) = mouse_to_world(q_camera, q_windows) {
            info!("Mouse clicked at world position: {:?}", world_position);
            for _ in 0..10 {
                commands.spawn(create_prey(
                    world_position.into(),
                    SpeciesId(0),
                    Genes::default(),
                ));
            }
        }
    } else if mouse_button_input.just_pressed(MouseButton::Right) {
        if let Some(world_position) = mouse_to_world(q_camera, q_windows) {
            info!(
                "Mouse right clicked at world position: {:?}",
                world_position
            );
            let count = rng.gen_range(10..30);
            for _ in 0..count {
                commands.spawn(create_food(
                    world_position
                        + Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0)),
                    rng.gen_range(10.0..100.0),
                ));
            }
        }
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
