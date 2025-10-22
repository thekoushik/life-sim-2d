use bevy::prelude::*;
use crate::entities::systems::{setup_entities, update_behaviors, eat_food,starve, update_hunger};
use crate::world::config::{save_config};
use crate::entities::components::{
    BehaviorState, EntityColor, EntityType, Hunger, Position, Velocity
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_entities)
        // app.add_systems(Startup, load_config)
            .add_systems(Update, (update_behaviors, update_hunger, eat_food, starve, save_on_keypress));
    }
}

fn save_on_keypress(
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(&Position, Option<&Velocity>, &EntityType, &EntityColor, Option<&Hunger>, Option<&BehaviorState>)>,
) {
    if input.just_pressed(KeyCode::KeyS) {
        save_config(query);
        info!("Saved simulation state to assets/save.ron");
    }
}