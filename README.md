# Life Sim 2D

This is a 2D life simulation built with Bevy.

## Installation
1. Clone the repository
2. Run `cargo run` to download dependencies and start the simulation
3. Press F12 to toggle the FPS counter(on by default)

## Features

- Prey(Yellow) and Food(Green) entities
- Spatial grid for efficient lookup
- Perception system for detecting nearby entities
- Behavior system for controlling entity behavior
- Genetic system
- Camera movement with WASD keys

## Requirements
- Bevy 0.14.0
- Rust 1.85.1

## TODO

- [ ] Needs system for tracking entity needs
- [ ] Add species system for grouping entities
- [ ] Add predators(Red) system for hunting prey
- [ ] Reproduction system for entities
- [ ] Simulation speed
- [ ] Add corpse and decay system
- [ ] Add more complex behavior
- [ ] Add more complex environment
- [ ] Add more complex interactions
- [ ] Add more complex ai
- [ ] Add more complex genetics
- [ ] Add more complex evolution
- [ ] Monitoring system for tracking entity health and behavior
- [ ] UI for managing simulation
