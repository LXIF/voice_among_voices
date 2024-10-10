pub mod audio;
mod physics;
mod structs;
#[cfg(test)]
pub mod test_functions;
mod utils;

use audio::*;
use ic_cdk::{api::performance_counter, init, post_upgrade, query, update};
use ic_cdk_timers::set_timer;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl, StableBTreeMap,
};
use once_cell::sync::Lazy;
use physics::*;
use serde_bytes::ByteBuf;
use std::{cell::RefCell, collections::HashMap, time::Duration};
use structs::*;
use utils::{node_within_circle, split_into_chunks};

thread_local! { // TODO: replace with stable structures and make auto-scaling
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static COLLIDER_COORDINATES: RefCell<Vec<ColliderCoordinate>> = RefCell::new(vec![]);
    static VOICE_NODES: RefCell<VoiceNodeLocalStore> = RefCell::new(vec![]);
    static SAMPLES_MAP: RefCell<StableBTreeMap<u128, AudioSample, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(0))))
    );
    static ANGLE_FILE_CACHE: RefCell<FileCache> = RefCell::new(HashMap::new());
    // static USERS: RefCell<UserStore> = RefCell::new(BTreeMap::new()); //TODO: check how init and pre/post upgrade affect this
    // static HISTORY: RefCell<Vec<HistoryFrame>> = RefCell::new(vec![]);
}

static STREAMING_CALLBACK: Lazy<CallbackFunc> =
    Lazy::new(|| CallbackFunc::new(ic_cdk::id(), "http_request_streaming_callback".to_string()));

const AUDIO_PARAMETERS: AudioParameters = AudioParameters {
    total_length_ms: 60 * 1000,
    max_sample_length_ms: 10000,
    sample_rate: 44100,
    chunk_size: 1024 * 1024,
    fade_ms: 30,
};
const SIMULATION_PARAMETERS: SimulationParameters = SimulationParameters {
    velocity_cutoff: 0.2,
    force_cutoff: 100.,
    max_distance: 20.,
    force_strength: 3000.,
    linear_damping: 10.,
    logical_radius: 50.,
    n_collider_vertices: 360,
    friction: 0.5,
    density: 2.,
};

// abstracting this because during dev things change and i don't want to restart dfx all the time
fn collider_init() {
    COLLIDER_COORDINATES.with_borrow_mut(|collider_coordinates| {
        let fresh_vertices = create_circular_collider_coordinates(
            SIMULATION_PARAMETERS.n_collider_vertices,
            SIMULATION_PARAMETERS.logical_radius,
        );

        collider_coordinates.extend(fresh_vertices);
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
    let (sample_length_samples, sample_length_ms) = get_sample_length(&node.sample)?;
    let max_sample_length = AUDIO_PARAMETERS.max_sample_length_ms;

    if sample_length_ms > max_sample_length as f64 {
        return Err(AddVoiceNodeError::NotValidAudioFileError(
            "Audio file too long".to_string(),
        ));
    }

    let node_radius = {
        let logical_per_ms =
            2. * SIMULATION_PARAMETERS.logical_radius / AUDIO_PARAMETERS.total_length_ms as f64;

        sample_length_ms * logical_per_ms / 2.
    };

    // check if we can accept le circle
    let within_circle = node_within_circle(&node, &SIMULATION_PARAMETERS, node_radius);

    if !within_circle {
        return Err(AddVoiceNodeError::NotWithinCircleError(
            "Node out of bounds".to_string(),
        ));
    };

    let mut sample_id = 0;

    SAMPLES_MAP.with_borrow_mut(|samples_map| {
        sample_id = samples_map.len();
        let new_sample = AudioSample {
            id: sample_id,
            sample: node.sample,
            sample_length_samples,
            sample_length_ms,
        };
        samples_map.insert(sample_id as u128, new_sample);
    });

    let mut returnable_nodes: VoiceNodeEgressStore = vec![];

    COLLIDER_COORDINATES.with_borrow(|collider_coordinates| {
        VOICE_NODES.with_borrow_mut(|nodes| {
            let id = nodes.len().into();
            let new_node = VoiceNodeLocal {
                id,
                x: node.x,
                y: node.y,
                sample_id,
                radius: node_radius,
                sample_length_samples,
            };

            nodes.push(new_node);

            simulate_until_stopped(nodes, &SIMULATION_PARAMETERS, &collider_coordinates);

            returnable_nodes = nodes.clone().into_iter().map(|node| node.into()).collect();
        });
    });

    Ok(returnable_nodes)
}

#[query]
fn get_voice_nodes() -> VoiceNodeEgressStore {
    VOICE_NODES.with_borrow(|nodes| nodes.clone().into_iter().map(|node| node.into()).collect())
}

#[query]
fn get_my_voice() -> Option<AudioSample> {
    //TODO: guard and return 'owned' sample
    SAMPLES_MAP.with_borrow(|samples_map| {
        if samples_map.len() >= 1 {
            Some(samples_map.get(&(0 as u128)).unwrap())
        } else {
            None
        }
    })
}

#[update]
fn get_angle_file(angle: f64) -> HttpStreamingResponse {
    // TODO: guard to user, restrict angle
    let beginning_cost = performance_counter(0);

    let mut result: Vec<u8> = vec![];

    VOICE_NODES.with_borrow(|nodes| {
        SAMPLES_MAP.with_borrow(|samples_map| {
            result = generate_angle_file(
                angle,
                nodes,
                samples_map,
                &AUDIO_PARAMETERS,
                &SIMULATION_PARAMETERS,
            )
            .unwrap(); // TODO: error handling
        });
    });

    // split into chunks
    let chunks = split_into_chunks(result, &AUDIO_PARAMETERS);

    ANGLE_FILE_CACHE.with_borrow_mut(|cache| {
        cache.insert(angle as u32, chunks.clone());
    });

    set_timer(Duration::from_secs(60), move || {
        // invalidate after 60s for now
        ANGLE_FILE_CACHE.with_borrow_mut(|cache| {
            cache.remove(&(angle as u32));
        });
    }); //invalidate after 60 seconds

    let total_chunks = chunks.len() as u32;

    let first_chunk = chunks.get(0).cloned().unwrap_or_default();
    let token = StreamingCallbackToken {
        angle: angle as u32,
        chunk_index: 0,
        chunks: total_chunks,
        auth_token: None, // TODO: maybe implement this for security purposes
    };

    let end_cost = performance_counter(0);

    HttpStreamingResponse {
        status_code: 200,
        headers: vec![
            ("content-type".to_string(), "audio/wav".to_string()),
            ("x-beginning-cost".to_string(), beginning_cost.to_string()), // Profiling header
            ("x-end-cost".to_string(), end_cost.to_string()),             // Profiling header
        ],
        body: ByteBuf::from(first_chunk),
        upgrade: None,
        streaming_strategy: create_strategy(token),
    }
}

#[query(hidden = true)]
fn http_request_streaming_callback(token: StreamingCallbackToken) -> StreamingCallbackHttpResponse {
    // TODO: perhaps make function naming more concise as this is only for getting angle files

    let beginning_cost = performance_counter(0);

    let chunks = match ANGLE_FILE_CACHE.with_borrow(|cache| cache.get(&token.angle).cloned()) {
        Some(file_chunks) => file_chunks,
        None => ic_cdk::trap("Cache out of date, connection too slow"),
    };

    if let Some(token) = token.next() {
        if let Some(chunk) = chunks.get((token.chunk_index) as usize) {
            let end_cost = performance_counter(0);
            StreamingCallbackHttpResponse {
                headers: vec![
                    ("x-beginning-cost".to_string(), beginning_cost.to_string()), // Profiling header
                    ("x-end-cost".to_string(), end_cost.to_string()), // Profiling header
                ],
                body: ByteBuf::from(chunk.clone()),
                token: Some(token),
            }
        } else {
            ic_cdk::trap("Chunk not found");
        }
    } else {
        let end_cost = performance_counter(0);
        StreamingCallbackHttpResponse {
            headers: vec![
                ("x-beginning-cost".to_string(), beginning_cost.to_string()), // Profiling header
                ("x-end-cost".to_string(), end_cost.to_string()),             // Profiling header
            ],
            body: ByteBuf::new(),
            token: None,
        }
    }
}

fn create_strategy(token: StreamingCallbackToken) -> Option<StreamingStrategy> {
    let callback = STREAMING_CALLBACK.clone();

    Some(StreamingStrategy::Callback { token, callback })
}

#[query]
fn get_simulation_parameters() -> SimulationParameters {
    SIMULATION_PARAMETERS.clone()
}

#[query]
fn get_collider_coordinates() -> Vec<ColliderCoordinate> {
    COLLIDER_COORDINATES.with_borrow(|coords| coords.clone())
}

#[query]
fn get_audio_parameters() -> AudioParameters {
    AUDIO_PARAMETERS.clone()
}

#[cfg(test)]
mod tests {
    use super::test_functions::*;
    use super::*;

    #[test]
    fn voice_nodes_get_added_correctly() {
        collider_init();

        let voice_node = VoiceNodeIngress {
            x: 0.,
            y: 40.,
            sample: generate_test_wav(1000, 44100),
        };

        let another_voice_node = VoiceNodeIngress {
            x: -45.,
            y: 0.,
            sample: generate_test_wav(1000, 44100),
        };

        let result_a = add_voice_node(voice_node);
        println!("{:#?}", result_a.unwrap());
        let _ = add_voice_node(another_voice_node);

        SAMPLES_MAP.with_borrow(|samples_map| {
            let id = samples_map.get(&0).expect("No sample!").id;
            let another_id = samples_map.get(&1).expect("No another_sample!").id;

            assert_eq!(id, 0);
            assert_eq!(another_id, 1);
        });
    }

    #[test]
    fn get_correct_audio_params() {
        let audio_params = get_audio_parameters();

        assert!(audio_params.max_sample_length_ms > 0);
    }

    #[test]
    fn out_of_bounds_voice_nodes_get_rejected_correctly() {
        collider_init();

        let voice_node = VoiceNodeIngress {
            x: -50.,
            y: -50.,
            sample: generate_test_wav(1000, 44100),
        };

        let another_voice_node = VoiceNodeIngress {
            x: 99.,
            y: 50.,
            sample: generate_test_wav(10000, 44100),
        };

        let _ = add_voice_node(voice_node);
        let _ = add_voice_node(another_voice_node);

        SAMPLES_MAP.with_borrow(|samples_map| {
            println!("{}", samples_map.len());
            assert!(samples_map.len() == 0);
        });
    }

    #[test]
    fn too_long_voice_nodes_get_rejected_correctly() {
        collider_init();

        let max_length = AUDIO_PARAMETERS.max_sample_length_ms;

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

        SAMPLES_MAP.with_borrow(|samples_map| {
            println!("{}", samples_map.len());
            assert!(samples_map.len() == 0);
        });
    }

    #[test]
    fn init_creates_coordinates() {
        init();
        COLLIDER_COORDINATES.with_borrow(|collider_coordinates| {
            let n = SIMULATION_PARAMETERS.n_collider_vertices;
            let len = collider_coordinates.len();

            assert_eq!(n, len as u64);
        });
    }
}
