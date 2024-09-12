use candid::CandidType;
use rapier2d::{parry::shape::Ball, prelude::*};

use crate::{SimulationParameters, VoiceNodeLocal, VoiceNodeLocalStore};

#[derive(Debug, Copy, Clone, CandidType)]
pub struct ColliderCoordinate {
    x: f64,
    y: f64,
}

struct PhysicsBody {
    collider_handle: ColliderHandle,
    rigid_body_handle: RigidBodyHandle,
    voice_node_id: usize,
}

/// simulates the new field until all bodies are at rest
pub fn simulate_until_stopped(
    nodes: &VoiceNodeLocalStore,
    parameters: &SimulationParameters,
    collider_coordinates: &Vec<ColliderCoordinate>,
) -> VoiceNodeLocalStore {
    // SETUP///////////////////////////////////////////////////

    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    let mut bodies: Vec<PhysicsBody>;

    // create collider
    let formatted_collider_coordinates: Vec<Point<Real>> = collider_coordinates
        .iter()
        .map(|coord| Point::new(coord.x as Real, coord.y as Real))
        .collect();

    let collider = ColliderBuilder::polyline(formatted_collider_coordinates, None);

    collider_set.insert(collider);

    // create rigid bodies and colliders for nodes

    bodies = nodes
        .iter()
        .map(|node| {
            let new_rigid_body = RigidBodyBuilder::dynamic()
                .translation(vector![node.x as f32, node.y as f32])
                .lock_rotations()
                .linear_damping(parameters.linear_damping as f32)
                .user_data(node.id as u128);

            let new_collider = ColliderBuilder::ball(2.)
                .density(2.)
                .friction(parameters.friction as f32); // TODO: replace with length and density from sample

            let rigid_body_handle = rigid_body_set.insert(new_rigid_body);
            let collider_handle = collider_set.insert_with_parent(
                new_collider,
                rigid_body_handle,
                &mut rigid_body_set,
            );

            PhysicsBody {
                voice_node_id: node.id,
                collider_handle,
                rigid_body_handle,
            }
        })
        .collect();

    // SIMULATE
    fn apply_magnetism_forces(
        bodies: Vec<PhysicsBody>,
        parameters: &SimulationParameters,
        rigid_body_set: RigidBodySet,
        collider_set: ColliderSet,
    ) {
        bodies.iter().for_each(|body| {
            let rigid_body = rigid_body_set.get(body.rigid_body_handle).unwrap();
            let collider = collider_set.get(body.collider_handle).unwrap();

            let cutoff_position = rigid_body.translation();
            let cutoff_rotation = rigid_body.rotation();
            let cutoff_shape = Ball::new(parameters.max_distance as f32);

            let bodies_within_reach: Vec<Collider>;
            let magnetic_forces: Vec<Vector<Real>>;

            // INTERSECTION TEST FROM QUERY PIPELINE
        });
    }

    todo!()
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
