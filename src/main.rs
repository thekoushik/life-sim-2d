use bevy::prelude::*;
mod plugins;
mod entities;
mod world;
use plugins::simulation::SimulationPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "2D Life Simulation".into(),
                    resolution: (800.0, 600.0).into(),
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
        .add_plugins(SimulationPlugin) // Custom simulation logic
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(500.0, 500.0, 0.0)), // Center on world
        ..default()
    });
}