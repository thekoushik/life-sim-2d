use crate::entities::components::{
    BehaviorState, EntityColor, Food, Needs, Position, Prey, SimulationSpeed, SpatialGrid,
};
use crate::entities::gameloop::{game_loop_predator, game_loop_prey};
use crate::entities::perception::{perception_scan_system_predator, perception_scan_system_prey};
use crate::entities::systems::{handle_input, setup_entities, update_entities, update_grid_system};
use crate::world::config::save_config;
use bevy::prelude::*;
const CAMERA_SPEED: f32 = 100.;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, load_config)
        app.add_systems(Startup, setup_entities)
            .insert_resource(SpatialGrid {
                cell_size: 64.0,
                ..Default::default()
            })
            .insert_resource(SimulationSpeed(2.0))
            // entity systems
            .add_systems(
                Update,
                (
                    update_grid_system,
                    perception_scan_system_prey,
                    perception_scan_system_predator,
                    game_loop_prey,
                    game_loop_predator,
                    update_entities,
                )
                    .chain(),
            )
            // input systems
            .add_systems(
                Update,
                (save_on_keypress, move_camera, handle_input).chain(),
            );
    }
}

fn save_on_keypress(
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(
        &Position,
        Option<&Food>,
        Option<&Prey>,
        &EntityColor,
        Option<&Needs>,
        Option<&BehaviorState>,
    )>,
) {
    if input.just_pressed(KeyCode::KeyX) {
        save_config(query);
        info!("Saved simulation state to assets/save.ron");
    }
}

fn move_camera(
    mut camera: Query<&mut Transform, With<Camera2d>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut camera_transform = camera.single_mut();
    let mut direction = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }
    let move_delta = direction.normalize_or_zero() * CAMERA_SPEED * time.delta_seconds();
    camera_transform.translation += move_delta.extend(0.);
}
