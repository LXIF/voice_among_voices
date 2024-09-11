use rapier2d::prelude::*;

use crate::{SimulationParameters, VoiceNodeLocalStore};

/// simulates the new field until all bodies are at rest
fn simulate_until_stopped(
    bodies: VoiceNodeLocalStore,
    parameters: SimulationParameters,
) -> VoiceNodeLocalStore {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    // create colliders

    todo!()
}

struct ColliderCoordinate {
    x: f64,
    y: f64,
}

/// generates circular coordinates at full width for the circular collider
fn create_circular_collider_coordinates(
    n_vertex: u64,
    logical_width: f64,
    logical_height: f64,
) -> Vec<ColliderCoordinate> {
    let mut coordinates: Vec<ColliderCoordinate> = Vec::with_capacity(n_vertex as usize);

    for i in 0..n_vertex {
        let angle: f64 = 2. * std::f64::consts::PI / n_vertex as f64 * i as f64;
        let x = logical_width / 2. * angle.cos() + logical_width / 2.;
        let y = logical_height / 2. * angle.sin() + logical_height / 2.;

        coordinates[i as usize] = ColliderCoordinate { x, y };
    }

    coordinates
}
