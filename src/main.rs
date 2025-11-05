use bevy::prelude::*;
mod plugins;
mod entities;
mod world;
mod helpers;
use plugins::simulation::SimulationPlugin;
use plugins::fps::FpsPlugin;
use helpers::util::{WORLD_WIDTH, WORLD_HEIGHT};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "2D Life Simulation".into(),
                    resolution: (WORLD_WIDTH, WORLD_HEIGHT).into(),
                    ..default()
                }),
                ..default()
            })
            .set(bevy::render::RenderPlugin {
                render_creation: bevy::render::settings::RenderCreation::Automatic(
                    bevy::render::settings::WgpuSettings {
                        backends: Some(bevy::render::settings::Backends::GL),
                        ..default()
                    },
                ),
                ..default()
            })
        )
        .add_plugins(FpsPlugin)
        .add_plugins(SimulationPlugin) // Custom simulation logic
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(WORLD_WIDTH / 2.0, WORLD_HEIGHT / 2.0, 0.0)), // Center on world
        ..default()
    });
}