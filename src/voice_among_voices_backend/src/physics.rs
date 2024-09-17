use crate::{SimulationParameters, VoiceNodeLocal, VoiceNodeLocalStore};
use candid::CandidType;
use futures::executor::block_on;
use nalgebra::{distance, Const, Isometry, Isometry2, OPoint, Point2, Vector2};
use rapier2d::{parry::shape::Ball, prelude::*};

#[derive(Debug, Copy, Clone, CandidType)]
pub struct ColliderCoordinate {
    x: f64,
    y: f64,
}

impl Into<OPoint<f32, Const<2>>> for ColliderCoordinate {
    fn into(self) -> OPoint<f32, Const<2>> {
        Point::new(self.x as f32, self.y as f32)
    }
}

#[derive(Debug)]
struct PhysicsBody {
    collider_handle: ColliderHandle,
    rigid_body_handle: RigidBodyHandle,
    voice_node_id: usize,
}

/// simulates the new field until all bodies are at rest
pub fn simulate_until_stopped(
    //mutates the store!
    nodes: &mut VoiceNodeLocalStore,
    parameters: &SimulationParameters,
    collider_coordinates: &Vec<ColliderCoordinate>,
) -> VoiceNodeLocalStore {
    // SETUP///////////////////////////////////////////////////

    let gravity = vector![0., 0.];
    let integration_parameters = IntegrationParameters::default();
    let mut island_manager = IslandManager::new();
    let mut broad_phase = DefaultBroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joint_set = ImpulseJointSet::new();
    let mut multibody_joint_set = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let physics_hooks = ();
    let event_handler = ();

    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();
    let mut query_pipeline = QueryPipeline::new();
    let mut physics_pipeline = PhysicsPipeline::new();

    let mut bodies: Vec<PhysicsBody>;

    // create world collider
    let formatted_collider_coordinates: Vec<Point<Real>> = collider_coordinates
        .iter()
        .map(|coord| Point::new(coord.x as Real, coord.y as Real))
        .collect();

    let collider = ColliderBuilder::polyline(formatted_collider_coordinates, None).build();

    collider_set.insert(collider);

    // create rigid bodies and colliders for nodes

    bodies = nodes
        .iter()
        .map(|node| {
            let new_rigid_body = RigidBodyBuilder::dynamic()
                .translation(vector![node.x as f32, node.y as f32])
                .lock_rotations()
                .linear_damping(parameters.linear_damping as f32)
                .user_data(node.id as u128)
                .build();

            let new_collider = ColliderBuilder::ball(2.)
                .density(2.)
                .friction(parameters.friction as f32)
                .build(); // TODO: replace with length and density from sample

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

    // DEBUG
    // for body in bodies.iter() {
    //     let collider_handle = body.collider_handle;

    //     let parent_body = collider_set.get(collider_handle).unwrap().parent().unwrap();
    //     println!("parent_body in setup: {:#?}", parent_body);
    // }

    // SIMULATE

    // apply force
    // step until no more forces

    let max_steps = 10_000;
    let mut steps = 0;

    let mut new_nodes: VoiceNodeLocalStore = vec![]; //TODO: remove this when no longer needed

    loop {
        steps += 1;
        apply_magnetism_forces(
            &mut bodies,
            parameters,
            &mut rigid_body_set,
            &collider_set,
            &query_pipeline,
        );
        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            &mut rigid_body_set,
            &mut collider_set,
            &mut impulse_joint_set,
            &mut multibody_joint_set,
            &mut ccd_solver,
            Some(&mut query_pipeline),
            &physics_hooks,
            &event_handler,
        );

        for physics_body in bodies.iter() {
            let position = rigid_body_set
                .get(physics_body.rigid_body_handle)
                .unwrap()
                .translation();
        }

        let still_moving = check_if_still_moving(&bodies, &rigid_body_set);

        // if !still_moving || steps >= max_steps {
        //     for physics_body in bodies.iter() {
        //         let position = rigid_body_set
        //             .get(physics_body.rigid_body_handle)
        //             .unwrap()
        //             .translation();
        //         let node_to_be_updated = nodes
        //             .iter_mut()
        //             .find(|node| node.id == physics_body.voice_node_id)
        //             .unwrap();

        //         node_to_be_updated.x = position[0].into();
        //         node_to_be_updated.y = position[1].into();
        //     }
        //     break;
        // }

        // FOR TESTING, LETS JUST RETURN THIS SO WE CAN READ IT OUT
        // for prod, might it make sense to mutate in-place?
        if (!still_moving && steps > 50) || steps >= max_steps {
            // if steps >= max_steps {
            for physics_body in bodies.iter() {
                let position = rigid_body_set
                    .get(physics_body.rigid_body_handle)
                    .unwrap()
                    .translation();
                let node_to_be_referenced = nodes
                    .iter()
                    .find(|node| node.id == physics_body.voice_node_id)
                    .unwrap();

                let new_node = VoiceNodeLocal {
                    id: node_to_be_referenced.id,
                    sample_id: node_to_be_referenced.sample_id,
                    x: position[0].into(),
                    y: position[1].into(),
                };
                new_nodes.push(new_node);

                // node_to_be_updated.x = position[0].into();
                // node_to_be_updated.y = position[1].into();
            }
            break;
        }
    }

    new_nodes
}

fn apply_magnetism_forces(
    bodies: &mut Vec<PhysicsBody>,
    parameters: &SimulationParameters,
    rigid_body_set: &mut RigidBodySet,
    collider_set: &ColliderSet,
    query_pipeline: &QueryPipeline,
) {
    for body in bodies.iter().rev() {
        let rigid_body = rigid_body_set.get(body.rigid_body_handle).unwrap();
        let collider = collider_set.get(body.collider_handle).unwrap();
        let cutoff_position = *rigid_body.position();
        let cutoff_shape = Ball::new(parameters.max_distance as f32);
        let filter = QueryFilter::default();

        let mut bodies_within_reach: Vec<ColliderHandle> = vec![];
        let mut magnetic_forces: Vec<Vector<Real>> = vec![];

        // INTERSECTION TEST FROM QUERY PIPELINE
        query_pipeline.intersections_with_shape(
            rigid_body_set,
            collider_set,
            &cutoff_position,
            &cutoff_shape,
            filter,
            |body_within_reach| {
                bodies_within_reach.push(body_within_reach);
                true
            },
        );

        let body_position = rigid_body.position();

        // create force vectors and add them to magnetic_forces
        for body_within_reach_collider_handle in bodies_within_reach.iter() {
            if body.collider_handle == *body_within_reach_collider_handle {
                continue;
            }

            if let Some(rigid_body_within_reach_handle) = collider_set
                .get(*body_within_reach_collider_handle)
                .and_then(|collider| collider.parent())
            {
                let body_within_reach = rigid_body_set.get(rigid_body_within_reach_handle).unwrap();
                let body_within_reach_pos = body_within_reach.position();

                let distance_between_bodies = distance(
                    &Point2::new(body_position.translation.x, body_position.translation.y),
                    &Point2::new(
                        body_within_reach_pos.translation.x,
                        body_within_reach_pos.translation.y,
                    ),
                );

                if distance_between_bodies > 0.0 {
                    let magnetic_force_scalar =
                        (1. / distance_between_bodies.powi(2)) * parameters.force_strength as f32;
                    let vector_between_bodies =
                        Vector2::new(body_position.translation.x, body_position.translation.y)
                            - Vector2::new(
                                body_within_reach_pos.translation.x,
                                body_within_reach_pos.translation.y,
                            );

                    // println!("Magnetic force scalar: {}", magnetic_force_scalar);
                    // println!("Vector between bodies: {:?}", vector_between_bodies);

                    magnetic_forces.push(Vector2::new(
                        vector_between_bodies.x * magnetic_force_scalar,
                        vector_between_bodies.y * magnetic_force_scalar,
                    ));
                }
            } else {
                // Handle World collider which doesn't have a parent
                continue;
            }
        }

        // reset forces
        let rigid_body = rigid_body_set.get_mut(body.rigid_body_handle).unwrap();
        rigid_body.reset_forces(true);

        let resultant_abs = magnetic_forces.iter().fold(0., |acc, force| {
            acc + distance(&Point::new(0., 0.), &Point::new(force.x, force.y))
        });

        if resultant_abs < parameters.force_cutoff as f32 {
            rigid_body.sleep();
            return;
        };

        for force in magnetic_forces.iter() {
            rigid_body.add_force(*force, true);
        }
    }
}

fn check_if_still_moving(bodies: &Vec<PhysicsBody>, rigid_body_set: &RigidBodySet) -> bool {
    let mut moving = false;
    for body in bodies.iter() {
        let rigid_body = rigid_body_set.get(body.rigid_body_handle).unwrap();
        if rigid_body.is_moving() {
            moving = true;
            break;
        }
    }
    moving
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

    use crate::SIMULATION_PARAMETERS;

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

    #[test]
    fn physics_sim_sanity_test() {
        let mut nodes: VoiceNodeLocalStore = vec![];
        let collider_coordinates = create_circular_collider_coordinates(360, 100., 100.);
        let mut result: VoiceNodeLocalStore = vec![];

        for i in 0..4 {
            let node = VoiceNodeLocal {
                id: i,
                x: 5. + 5. * i as f64,
                y: 50.,
                sample_id: i,
            };

            nodes.push(node);
        }

        SIMULATION_PARAMETERS.with(|parameters| {
            result = simulate_until_stopped(&mut nodes, parameters, &collider_coordinates);
        });

        println!("{:#?}", result);
        assert!(result[0].x > 0.);
        assert!(result[0].x < 100.);
    }

    #[test]
    fn physics_moves_bodies() {
        let mut nodes: VoiceNodeLocalStore = vec![];
        let collider_coordinates = create_circular_collider_coordinates(360, 100., 100.);
        let mut result: VoiceNodeLocalStore = vec![];

        let node_a = VoiceNodeLocal {
            id: 0,
            x: 48.,
            y: 50.,
            sample_id: 0,
        };

        let node_b = VoiceNodeLocal {
            id: 1,
            x: 52.,
            y: 50.,
            sample_id: 1,
        };

        nodes.push(node_a);
        nodes.push(node_b);

        SIMULATION_PARAMETERS.with(|parameters| {
            result = simulate_until_stopped(&mut nodes, parameters, &collider_coordinates);
        });

        println!("{:#?}", result);
        assert!(result[0].x < 48.);
        assert!(result[1].x > 52.);
    }
}
