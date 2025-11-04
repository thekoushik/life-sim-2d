use bevy::prelude::*;
use crate::entities::systems::{setup_entities, game_loop};
use crate::world::config::{save_config};
use crate::entities::components::{
    BehaviorState, EntityColor, Food, Prey, Hunger, Position, Velocity
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_entities)
        // app.add_systems(Startup, load_config)
            .add_systems(Update, (game_loop, save_on_keypress));
    }
}

fn save_on_keypress(
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(&Position, Option<&Velocity>, Option<&Food>, Option<&Prey>, &EntityColor, Option<&Hunger>, Option<&BehaviorState>)>,
) {
    if input.just_pressed(KeyCode::KeyS) {
        save_config(query);
        info!("Saved simulation state to assets/save.ron");
    }
}