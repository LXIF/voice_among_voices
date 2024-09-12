use candid::CandidType;
use rapier2d::prelude::*;

use crate::{SimulationParameters, VoiceNodeLocalStore};

/// simulates the new field until all bodies are at rest
pub fn simulate_until_stopped(
    bodies: VoiceNodeLocalStore,
    parameters: SimulationParameters,
) -> VoiceNodeLocalStore {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    // create colliders

    todo!()
}

#[derive(Debug, Copy, Clone, CandidType)]
pub struct ColliderCoordinate {
    x: f64,
    y: f64,
}

/// generates circular coordinates at full width for the circular collider
pub fn create_circular_collider_coordinates(
    n_vertex: u64,
    logical_width: f64,
    logical_height: f64,
) -> Vec<ColliderCoordinate> {
    let mut coordinates: Vec<ColliderCoordinate> = Vec::with_capacity(n_vertex as usize);

    for i in 0..n_vertex {
        let angle: f64 = 2. * std::f64::consts::PI / n_vertex as f64 * i as f64;
        let x = logical_width / 2. * angle.cos() + logical_width / 2.;
        let y = logical_height / 2. * angle.sin() + logical_height / 2.;

        coordinates.push(ColliderCoordinate { x, y });
    }

    coordinates
}

#[cfg(test)]
mod tests {
    use super::*;

    // needed because floats and trigo aren't perfect lol
    fn approximately_equal(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn correct_number_of_circle_coordinates() {
        let n = 10;
        let result = create_circular_collider_coordinates(n, 100., 100.);
        assert_eq!(n, result.len() as u64);
    }

    #[test]
    fn correct_maximum_extents() {
        let n = 4;
        let result = create_circular_collider_coordinates(n, 100., 100.);
        let epsilon = 1e-6;
        // right
        assert!(approximately_equal(result[0].x, 100., epsilon));
        assert!(approximately_equal(result[0].y, 50., epsilon));

        // bottom
        assert!(approximately_equal(result[1].x, 50., epsilon));
        assert!(approximately_equal(result[1].y, 100., epsilon));

        // left
        assert!(approximately_equal(result[2].x, 0., epsilon));
        assert!(approximately_equal(result[2].y, 50., epsilon));

        // top
        assert!(approximately_equal(result[3].x, 50., epsilon));
        assert!(approximately_equal(result[3].y, 0., epsilon));
    }
}
