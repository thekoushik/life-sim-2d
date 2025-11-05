use bevy::prelude::*;
use ron::from_str;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::entities::components::{Position, Prey, Food, EntityColor, Hunger, BehaviorState};

#[derive(Serialize, Deserialize)]
struct SimConfig {
    entities: Vec<EntityConfig>,
}

#[derive(Serialize, Deserialize)]
struct EntityConfig {
    position: (f32, f32),
    entity_type: String,
    color: (f32, f32, f32, f32),
    hunger: Option<f32>, // Optional for Prey only
    behavior_state: Option<String>, // Optional for Food
}

pub fn load_config(mut commands: Commands) {
    let config_path = "assets/save.ron";
    match fs::read_to_string(config_path) {
        Ok(config_str) => {
            match from_str::<SimConfig>(&config_str) {
                Ok(config) => {
                    for entity in config.entities {
                        let pos = Vec2::new(entity.position.0, entity.position.1);
                        // let vel = Vec2::new(entity.velocity.0, entity.velocity.1);
                        let mut entity_commands = commands.spawn((
                            Position(pos),
                            // Velocity(vel),
                            EntityColor(Color::srgba(entity.color.0, entity.color.1, entity.color.2, entity.color.3)),
                            SpriteBundle {
                                sprite: Sprite {
                                    color: Color::srgba(entity.color.0, entity.color.1, entity.color.2, entity.color.3),
                                    // custom_size: Some(Vec2::new(10.0, 10.0)),
                                    custom_size: Some(Vec2::new(2.0, 2.0)),
                                    ..default()
                                },
                                transform: Transform::from_translation(pos.extend(0.0)),
                                ..default()
                            },
                        ));
                        if let Some(hunger) = entity.hunger {
                            entity_commands.insert(Hunger(hunger));
                        }
                        if entity.entity_type.as_str() == "Food" {
                            entity_commands.insert(Food);
                        }
                        if entity.entity_type.as_str() == "Prey" {
                            entity_commands.insert(Prey);
                        }
                        if let Some(state) = entity.behavior_state {
                            let behavior_state = match state.as_str() {
                                "SeekFood" => BehaviorState::SeekFood,
                                "Sleep" => BehaviorState::Sleep,
                                // "InfluencedWork" => BehaviorState::InfluencedWork,
                                _ => {
                                    warn!("Unknown behavior state '{}', defaulting to Sleep", state);
                                    BehaviorState::Sleep
                                }
                            };
                            entity_commands.insert(behavior_state);
                        }
                    }
                    // info!("Loaded {} entities from {}", config.entities.len(), config_path);
                }
                Err(e) => {
                    warn!("Failed to parse config '{}': {}. Falling back to default initialization.", config_path, e);
                }
            }
        }
        Err(e) => {
            warn!("Failed to read config '{}': {}. Falling back to default initialization.", config_path, e);
        }
    }
}

pub fn save_config(query: Query<(&Position, Option<&Food>, Option<&Prey>, &EntityColor, Option<&Hunger>, Option<&BehaviorState>)>) {
    let entities: Vec<EntityConfig> = query
        .iter()
        .map(|(pos, _food, prey, color, hunger, behavior_state)| {
            let (r, g, b, a) = match color.0 {
                Color::Srgba(Srgba { red, green, blue, alpha, .. }) => {
                    (red, green, blue, alpha)
                },
                _ => (0.0, 0.0, 0.0, 1.0),
            };
            let entity_type = if prey.is_some() {
                "Prey".to_string()
            } else {
                "Food".to_string()
            };
            EntityConfig {
                position: (pos.0.x, pos.0.y),
                entity_type,
                color: (r, g, b, a),
                hunger: hunger.map(|h| h.0),
                behavior_state: behavior_state.map(|s| match s {
                    BehaviorState::SeekFood => "SeekFood".to_string(),
                    BehaviorState::Sleep => "Sleep".to_string(),
                    // BehaviorState::Flee => "Flee".to_string(),
                    BehaviorState::Wander => "Wander".to_string(),
                }),
            }
        })
        .collect();
    let config = SimConfig { entities };
    let ron_str = ron::to_string(&config).expect("Failed to serialize config");
    fs::write("assets/save.ron", ron_str).expect("Failed to write save file");
}