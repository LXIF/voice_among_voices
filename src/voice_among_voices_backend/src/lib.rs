mod physics;
mod utils;

use candid::{CandidType, Principal};
use futures::executor::block_on;
use ic_cdk::{api::time, init, post_upgrade, query, update};
use physics::*;
use serde::Deserialize;
use std::{cell::RefCell, collections::BTreeMap};
use utils::node_within_circle;

#[derive(Debug)]
struct User {
    voice_node: VoiceNodeLocal,
    signup_timestamp: u64,
}

type UserStore = BTreeMap<Principal, User>;

#[derive(Clone, Debug, Deserialize, CandidType)]
struct VoiceNodeIngress {
    x: f64,
    y: f64,
    sample: String, // TODO: update with audio type when we get to it. probably use hound crate. Might make sense to keep audio files separately or only return positions to FE + compiled audio
}

#[derive(Clone, Debug, Deserialize, CandidType)]
struct VoiceNodeEgress {
    id: usize,
    x: f64,
    y: f64,
}

impl From<VoiceNodeLocal> for VoiceNodeEgress {
    fn from(local: VoiceNodeLocal) -> Self {
        VoiceNodeEgress {
            id: local.id,
            x: local.x,
            y: local.y,
        }
    }
}

#[derive(Clone, Debug, CandidType)]
struct VoiceNodeLocal {
    id: usize,
    x: f64,
    y: f64,
    sample_id: usize,
}

type VoiceNodeLocalStore = Vec<VoiceNodeLocal>;
type VoiceNodeEgressStore = Vec<VoiceNodeEgress>;

#[derive(Debug, CandidType)]
struct AudioSample {
    id: usize,
    sample: String, // TODO: replace this with audio type
}

type AudioSampleStore = Vec<AudioSample>;

#[derive(Debug)]
struct NFTMap; // TODO: this is one of the last things to implement to make the whole thing NFT-compliant.

#[derive(Debug)]
struct HistoryFrame {
    timestamp: u64,
    nodes_states: Vec<VoiceNodeLocal>,
}

#[derive(Debug, Clone, Copy, CandidType)]
struct SimulationParameters {
    velocity_cutoff: f64,
    force_cutoff: f64,
    max_distance: f64,
    force_strength: f64,
    linear_damping: f64,
    logical_width: f64,
    logical_height: f64,
    n_collider_vertices: u64,
    friction: f64,
}

#[derive(CandidType, Debug)]
struct NotWithinCircleError;

thread_local! { // TODO: replace with stable structures and make auto-scaling
    static USERS: RefCell<UserStore> = RefCell::new(BTreeMap::new()); //TODO: check how init and pre/post upgrade affect this
    static VOICE_NODES: RefCell<VoiceNodeLocalStore> = RefCell::new(vec![]);
    static SAMPLES: RefCell<AudioSampleStore> = RefCell::new(vec![]);
    static HISTORY: RefCell<Vec<HistoryFrame>> = RefCell::new(vec![]);
    static SIMULATION_PARAMETERS: SimulationParameters = SimulationParameters {
        velocity_cutoff: 0.2,
        force_cutoff: 100.,
        max_distance: 20.,
        force_strength: 3000.,
        linear_damping: 10.,
        logical_height: 100.,
        logical_width: 100.,
        n_collider_vertices: 360,
        friction: 0.5
    };
    static COLLIDER_COORDINATES: RefCell<Vec<ColliderCoordinate>> = RefCell::new(vec![]);
}

// abstracting this because during dev things change and i don't want to restart dfx all the time
fn collider_init() {
    SIMULATION_PARAMETERS.with(|simulation_parameters| {
        COLLIDER_COORDINATES.with(|collider_coordinates| {
            let fresh_vertices = create_circular_collider_coordinates(
                simulation_parameters.n_collider_vertices,
                simulation_parameters.logical_width,
                simulation_parameters.logical_height,
            );

            collider_coordinates.borrow_mut().extend(fresh_vertices);
        });
    });
}

#[init]
fn init() {
    collider_init();
}

#[post_upgrade]
fn post_upgrade() {
    collider_init();
}

#[update]
fn add_voice_node(node: VoiceNodeIngress) -> Result<VoiceNodeEgressStore, NotWithinCircleError> {
    // check if we can accept le circle
    let within_circle =
        SIMULATION_PARAMETERS.with(|sim_params| node_within_circle(&node, sim_params));

    if !within_circle {
        return Err(NotWithinCircleError);
    };

    let mut sample_id = 0;
    SAMPLES.with(|samples| {
        let new_sample = AudioSample {
            id: samples.borrow().len(),
            sample: node.sample,
        };
        sample_id = new_sample.id.clone();
        samples.borrow_mut().push(new_sample);
    });

    let mut returnable_nodes: VoiceNodeEgressStore = vec![];

    SIMULATION_PARAMETERS.with(|parameters| {
        COLLIDER_COORDINATES.with(|collider_coordinates| {
            VOICE_NODES.with(|nodes| {
                let id = nodes.borrow().len().into();
                let new_node = VoiceNodeLocal {
                    id,
                    x: node.x,
                    y: node.y,
                    sample_id,
                };

                nodes.borrow_mut().push(new_node);

                returnable_nodes = simulate_until_stopped(
                    &mut nodes.borrow_mut(),
                    parameters,
                    &collider_coordinates.borrow(),
                )
                .into_iter()
                .map(|node| node.into())
                .collect()
            });
        });
    });

    Ok(returnable_nodes)
}

#[query]
fn voice_nodes() -> VoiceNodeEgressStore {
    VOICE_NODES.with(|nodes| {
        nodes
            .borrow()
            .clone()
            .into_iter()
            .map(|node| node.into())
            .collect()
    })
}

#[query]
fn get_simulation_parameters() -> SimulationParameters {
    SIMULATION_PARAMETERS.with(|params| params.clone())
}

#[query]
fn get_collider_coordinates() -> Vec<ColliderCoordinate> {
    COLLIDER_COORDINATES.with(|coords| coords.borrow().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_nodes_get_added_correctly() {
        collider_init();

        let voice_node = VoiceNodeIngress {
            x: 50.,
            y: 90.,
            sample: "wargle".to_string(),
        };

        let another_voice_node = VoiceNodeIngress {
            x: 5.,
            y: 50.,
            sample: "wargle".to_string(),
        };

        let result_a = add_voice_node(voice_node);
        println!("{:#?}", result_a.unwrap());
        let _ = add_voice_node(another_voice_node);

        SAMPLES.with(|samples| {
            let id = samples.borrow()[0].id;
            let another_id = samples.borrow()[1].id;

            assert_eq!(id, 0);
            assert_eq!(another_id, 1);
        });
    }

    #[test]
    fn voice_nodes_get_rejected_correctly() {
        let voice_node = VoiceNodeIngress {
            x: 0.,
            y: 0.,
            sample: "wargle".to_string(),
        };

        let another_voice_node = VoiceNodeIngress {
            x: 99.,
            y: 50.,
            sample: "wargle".to_string(),
        };

        let _ = add_voice_node(voice_node);
        let _ = add_voice_node(another_voice_node);

        SAMPLES.with(|samples| {
            println!("{}", samples.borrow().len());
            assert!(samples.borrow().len() == 0);
        });
    }

    #[test]
    fn init_creates_coordinates() {
        init();
        SIMULATION_PARAMETERS.with(|simulation_parameters| {
            COLLIDER_COORDINATES.with(|collider_coordinates| {
                let n = simulation_parameters.n_collider_vertices;
                let len = collider_coordinates.borrow().len();

                assert_eq!(n, len as u64);
            });
        });
    }
}
