mod audio;
mod physics;
mod utils;

use audio::*;
use candid::{CandidType, Principal};
use hound;
use ic_cdk::{api::time, init, post_upgrade, query, update};
use physics::*;
use serde::Deserialize;
use std::{borrow::Borrow, cell::RefCell, collections::BTreeMap};
use utils::{node_within_circle, sample_length_to_radius};

#[derive(Debug)]
struct User {
    voice_node_id: usize,
    signup_timestamp: u64,
}

type UserStore = BTreeMap<Principal, User>;

#[derive(Clone, Debug, Deserialize, CandidType)]
struct VoiceNodeIngress {
    x: f64,
    y: f64,
    sample: Vec<u8>, // here it's still a blob
}

#[derive(Clone, Debug, Deserialize, CandidType)]
struct VoiceNodeEgress {
    id: usize,
    x: f64,
    y: f64,
    radius: f64,
}

impl From<VoiceNodeLocal> for VoiceNodeEgress {
    fn from(local: VoiceNodeLocal) -> Self {
        VoiceNodeEgress {
            id: local.id,
            x: local.x,
            y: local.y,
            radius: local.radius,
        }
    }
}

#[derive(Clone, Debug, CandidType)]
struct VoiceNodeLocal {
    id: usize,
    x: f64,
    y: f64,
    sample_id: usize,
    radius: f64,
}

type VoiceNodeLocalStore = Vec<VoiceNodeLocal>;
type VoiceNodeEgressStore = Vec<VoiceNodeEgress>;

#[derive(Debug, CandidType, Clone)]
struct AudioSample {
    id: usize,
    sample: Vec<u8>,
    sample_length_ms: f64,
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
    density: f64,
}

#[derive(Debug, Clone, Copy, CandidType)]
struct AudioParameters {
    total_length_ms: u32,
    max_sample_length_ms: u32,
}

#[derive(CandidType, Debug)]
enum AddVoiceNodeError {
    NotWithinCircleError,
    NotValidAudioFileError(String),
}

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
        friction: 0.5,
        density: 2.
    };
    static COLLIDER_COORDINATES: RefCell<Vec<ColliderCoordinate>> = RefCell::new(vec![]);
    static AUDIO_PARAMETERS: AudioParameters = AudioParameters {
        total_length_ms: 60 * 1000,
        max_sample_length_ms: 10000,
    };
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
fn add_voice_node(node: VoiceNodeIngress) -> Result<VoiceNodeEgressStore, AddVoiceNodeError> {
    // first check radius
    let sample_length_ms = get_sample_length(&node.sample)?;
    let max_sample_length =
        AUDIO_PARAMETERS.with(|audio_params| audio_params.borrow().max_sample_length_ms);

    if sample_length_ms > max_sample_length as f64 {
        return Err(AddVoiceNodeError::NotValidAudioFileError(
            "Audio file too long".to_string(),
        ));
    }

    let node_radius = SIMULATION_PARAMETERS.with(|sim_params| {
        AUDIO_PARAMETERS.with(|audio_params| {
            let logical_per_ms = sim_params.logical_width / audio_params.total_length_ms as f64;

            sample_length_ms * logical_per_ms / 2.
        })
    });

    // check if we can accept le circle
    let within_circle =
        SIMULATION_PARAMETERS.with(|sim_params| node_within_circle(&node, sim_params, node_radius));

    if !within_circle {
        return Err(AddVoiceNodeError::NotWithinCircleError);
    };

    let mut sample_id = 0;
    SAMPLES.with(|samples| {
        let new_sample = AudioSample {
            id: samples.borrow().len(),
            sample: node.sample,
            sample_length_ms,
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
                    radius: node_radius,
                };

                nodes.borrow_mut().push(new_node);

                let simulated_nodes = simulate_until_stopped(
                    &nodes.borrow(),
                    parameters,
                    &collider_coordinates.borrow(),
                );

                for node in nodes.borrow_mut().iter_mut() {
                    if let Some(new_node) = simulated_nodes
                        .iter()
                        .find(|simulated_node| simulated_node.id == node.id)
                    {
                        node.x = new_node.x;
                        node.y = new_node.y;
                    }
                }

                returnable_nodes = simulated_nodes
                    .into_iter()
                    .map(|node| node.into())
                    .collect();
            });
        });
    });

    Ok(returnable_nodes)
}

#[query]
fn get_voice_nodes() -> VoiceNodeEgressStore {
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
fn get_my_voice() -> Option<AudioSample> {
    //TODO: guard and return 'owned' sample
    SAMPLES.with(|samples| {
        if samples.borrow().len() >= 1 {
            Some(samples.borrow()[0].clone())
        } else {
            None
        }
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

#[query]
fn get_audio_parameters() -> AudioParameters {
    AUDIO_PARAMETERS.with(|params| params.clone())
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
            sample: generate_test_wav(1000, 44100),
        };

        let another_voice_node = VoiceNodeIngress {
            x: 5.,
            y: 50.,
            sample: generate_test_wav(1000, 44100),
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

    // #[test]
    // fn voice_nodes_have_correct_radius() {
    //     collider_init();

    //     let voice_node = VoiceNodeIngress {
    //         x: 50.,
    //         y: 50.,
    //         sample: generate_test_wav(1000, 44100),
    //     };
    //     let sample_length = get_sample_length(&voice_node.sample).unwrap();
    //     let node_radius = SIMULATION_PARAMETERS.with(|sim_params| {
    //         AUDIO_PARAMETERS.with(|audio_params| {
    //             sample_length_to_radius(sample_length, sim_params, audio_params)
    //         })
    //     });

    //     let another_voice_node = VoiceNodeIngress {
    //         x: 60.,
    //         y: 60.,
    //         sample: generate_test_wav(2500, 44100),
    //     };

    //     let another_sample_length = get_sample_length(&another_voice_node.sample).unwrap();
    //     let another_node_radius = SIMULATION_PARAMETERS.with(|sim_params| {
    //         AUDIO_PARAMETERS.with(|audio_params| {
    //             sample_length_to_radius(another_sample_length, sim_params, audio_params)
    //         })
    //     });

    //     let result_a = add_voice_node(voice_node);
    //     println!("{:#?}", result_a.unwrap());
    //     let _ = add_voice_node(another_voice_node);

    //     VOICE_NODES.with(|voice_nodes| {
    //         SAMPLES.with(|samples| {
    //             let radius = voice_nodes.borrow()[0].radius;
    //             let sample_radius = get_sample_length(&samples.borrow()[0].sample).unwrap();

    //             let another_radius = voice_nodes.borrow()[1].radius;
    //             let another_sample = samples.borrow()[1];
    //         });
    //     });
    // }
    #[test]
    fn get_correct_audio_params() {
        let audio_params = get_audio_parameters();

        assert!(audio_params.max_sample_length_ms > 0);
    }

    #[test]
    fn out_of_bounds_voice_nodes_get_rejected_correctly() {
        collider_init();

        let voice_node = VoiceNodeIngress {
            x: 0.,
            y: 0.,
            sample: generate_test_wav(1000, 44100),
        };

        let another_voice_node = VoiceNodeIngress {
            x: 99.,
            y: 50.,
            sample: generate_test_wav(10000, 44100),
        };

        let _ = add_voice_node(voice_node);
        let _ = add_voice_node(another_voice_node);

        SAMPLES.with(|samples| {
            println!("{}", samples.borrow().len());
            assert!(samples.borrow().len() == 0);
        });
    }

    #[test]
    fn too_long_voice_nodes_get_rejected_correctly() {
        collider_init();

        let max_length = AUDIO_PARAMETERS.with(|params| params.max_sample_length_ms);

        let voice_node = VoiceNodeIngress {
            x: 50.,
            y: 50.,
            sample: generate_test_wav(max_length + 10, 44100),
        };

        let another_voice_node = VoiceNodeIngress {
            x: 60.,
            y: 60.,
            sample: generate_test_wav(max_length + 20, 44100),
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
