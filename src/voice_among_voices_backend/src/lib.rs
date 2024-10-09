mod audio;
mod physics;
mod utils;

use audio::*;
use candid::{define_function, CandidType, Principal};
use ic_cdk::{api::performance_counter, init, post_upgrade, query, update};
use ic_http_certification::HeaderField;
use once_cell::sync::Lazy;
use physics::*;
use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::{borrow::Borrow, cell::RefCell, collections::BTreeMap};
use utils::{node_within_circle, split_into_chunks};

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
    sample_length_samples: u32,
}

type AudioSampleStore = Vec<AudioSample>;

#[derive(Debug)]
struct NFTMap; // TODO: this is one of the last things to implement to make the whole thing NFT-compliant.

#[derive(Debug)]
struct HistoryFrame {
    timestamp: u64,
    nodes_states: Vec<VoiceNodeLocal>,
}

#[derive(Debug, Clone, Copy, CandidType, Deserialize)]
struct SimulationParameters {
    velocity_cutoff: f64,
    force_cutoff: f64,
    max_distance: f64,
    force_strength: f64,
    linear_damping: f64,
    logical_radius: f64,
    n_collider_vertices: u64,
    friction: f64,
    density: f64,
}

#[derive(Debug, Clone, Copy, CandidType)]
struct AudioParameters {
    total_length_ms: u32,
    max_sample_length_ms: u32,
    sample_rate: u32,
    chunk_size: usize,
    fade_ms: u32,
}

#[derive(CandidType, Debug)]
enum AddVoiceNodeError {
    NotWithinCircleError(String),
    NotValidAudioFileError(String),
}

#[derive(CandidType, Deserialize, Clone, Default)]
pub struct HttpStreamingResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub upgrade: Option<bool>,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StreamingCallbackToken {
    pub angle: u32,
    pub chunk_index: u32,
    pub chunks: u32,
    pub auth_token: Option<ByteBuf>,
}

impl StreamingCallbackToken {
    pub fn next(self) -> Option<StreamingCallbackToken> {
        if self.chunk_index + 1 >= self.chunks {
            None
        } else {
            Some(StreamingCallbackToken {
                angle: self.angle,
                chunk_index: self.chunk_index + 1,
                chunks: self.chunks,
                auth_token: self.auth_token,
            })
        }
    }
}

define_function!(pub CallbackFunc : (StreamingCallbackToken) -> (StreamingCallbackHttpResponse) query);

#[derive(CandidType, Deserialize, Clone)]
pub enum StreamingStrategy {
    Callback {
        token: StreamingCallbackToken,
        callback: CallbackFunc,
    },
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StreamingCallbackHttpResponse {
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub token: Option<StreamingCallbackToken>,
}

thread_local! { // TODO: replace with stable structures and make auto-scaling
    static USERS: RefCell<UserStore> = RefCell::new(BTreeMap::new()); //TODO: check how init and pre/post upgrade affect this
    static VOICE_NODES: RefCell<VoiceNodeLocalStore> = RefCell::new(vec![]);
    static SAMPLES: RefCell<AudioSampleStore> = RefCell::new(vec![]);
    static HISTORY: RefCell<Vec<HistoryFrame>> = RefCell::new(vec![]);
    static COLLIDER_COORDINATES: RefCell<Vec<ColliderCoordinate>> = RefCell::new(vec![]);
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
    COLLIDER_COORDINATES.with(|collider_coordinates| {
        let fresh_vertices = create_circular_collider_coordinates(
            SIMULATION_PARAMETERS.n_collider_vertices,
            SIMULATION_PARAMETERS.logical_radius,
        );

        collider_coordinates.borrow_mut().extend(fresh_vertices);
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
    SAMPLES.with(|samples| {
        let new_sample = AudioSample {
            id: samples.borrow().len(),
            sample: node.sample,
            sample_length_samples,
            sample_length_ms,
        };
        sample_id = new_sample.id.clone();
        samples.borrow_mut().push(new_sample);
    });

    let mut returnable_nodes: VoiceNodeEgressStore = vec![];

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

            simulate_until_stopped(
                &mut nodes.borrow_mut(),
                &SIMULATION_PARAMETERS,
                &collider_coordinates.borrow(),
            );

            returnable_nodes = nodes
                .borrow()
                .clone()
                .into_iter()
                .map(|node| node.into())
                .collect();
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
fn get_angle_file(angle: f64) -> HttpStreamingResponse {
    // TODO: guard to user, restrict angle
    // TODO: maybe store the file, maybe another for historical files
    let beginning_cost = performance_counter(0);

    let mut result: Vec<u8> = vec![];

    VOICE_NODES.with(|nodes| {
        SAMPLES.with(|samples| {
            result = generate_angle_file(
                angle,
                &*nodes.borrow(),
                &*samples.borrow(),
                &AUDIO_PARAMETERS,
                &SIMULATION_PARAMETERS,
            )
            .unwrap(); // TODO: error handling
        });
    });

    let chunks = split_into_chunks(result, &AUDIO_PARAMETERS);
    let total_chunks = chunks.len() as u32;

    let first_chunk = chunks.get(0).cloned().unwrap_or_default();
    let token = StreamingCallbackToken {
        angle: angle as u32, // TODO: look at this again, esp type casting - should work because nfts are for full degrees
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

    // TODO: perhaps cache the file

    let beginning_cost = performance_counter(0);
    let mut result: Vec<u8> = vec![];

    VOICE_NODES.with(|nodes| {
        SAMPLES.with(|samples| {
            result = generate_angle_file(
                token.angle as f64,
                &*nodes.borrow(),
                &*samples.borrow(),
                &AUDIO_PARAMETERS,
                &SIMULATION_PARAMETERS,
            )
            .unwrap(); // TODO: error handling
        });
    });

    let chunks = split_into_chunks(result, &AUDIO_PARAMETERS);

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
    COLLIDER_COORDINATES.with(|coords| coords.borrow().clone())
}

#[query]
fn get_audio_parameters() -> AudioParameters {
    AUDIO_PARAMETERS.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Decode;
    use pocket_ic::{PocketIc, WasmResult};
    use std::{fs::read, path::PathBuf};

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

        SAMPLES.with(|samples| {
            let id = samples.borrow()[0].id;
            let another_id = samples.borrow()[1].id;

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

        SAMPLES.with(|samples| {
            println!("{}", samples.borrow().len());
            assert!(samples.borrow().len() == 0);
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

        SAMPLES.with(|samples| {
            println!("{}", samples.borrow().len());
            assert!(samples.borrow().len() == 0);
        });
    }

    #[test]
    fn init_creates_coordinates() {
        init();
        COLLIDER_COORDINATES.with(|collider_coordinates| {
            let n = SIMULATION_PARAMETERS.n_collider_vertices;
            let len = collider_coordinates.borrow().len();

            assert_eq!(n, len as u64);
        });
    }

    // would need to update with pocket-ic
    // #[test]
    // fn streaming_sequencing_is_correct() {
    //     init();

    //     let token = StreamingCallbackToken {
    //         angle: 0,
    //         chunk_index: 0,
    //         chunks: 11,
    //         auth_token: None,
    //     };

    //     let response: StreamingCallbackHttpResponse = http_request_streaming_callback(token);

    //     println!("{:#?}", response.token);
    //     assert_eq!(response.token.unwrap().chunk_index, 1);
    // }

    fn pic_initialize_canister(pic: &PocketIc) -> Principal {
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);

        let mut wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        wasm_path.pop();
        wasm_path.pop();
        wasm_path.push(
            ".dfx/local/canisters/voice_among_voices_backend/voice_among_voices_backend.wasm",
        );
        println!("{:?}", wasm_path);

        let wasm_bytes = read(wasm_path).expect("failed to read wasm");

        pic.install_canister(canister_id, wasm_bytes, vec![], None);

        canister_id
    }

    #[test]
    fn pocket_ic_get_sim_params() {
        let pic = PocketIc::new();
        let canister_id = pic_initialize_canister(&pic);

        let result = pic
            .query_call(
                canister_id,
                Principal::anonymous(),
                "get_simulation_parameters",
                candid::encode_one(()).unwrap(),
            )
            .expect("failed to call canister");

        match result {
            WasmResult::Reply(bytes) => {
                let decoded_result = Decode!(&bytes, SimulationParameters);

                match decoded_result {
                    Ok(sim_params) => {
                        println!("Simulation Parameters: {:?}", sim_params);
                        assert!(true);
                    }
                    Err(e) => {
                        println!("failed to decode: {:?}", e);
                        assert!(false);
                    }
                }
            }
            WasmResult::Reject(error) => {
                println!("query rejected: {:?}", error);
                assert!(false);
            }
        }
    }

    #[test]
    fn pocket_ic_sample_to_angle_file_single() {
        let pic = PocketIc::new();
        let canister_id = pic_initialize_canister(&pic);

        // Generate a test WAV audio file
        let test_wav = generate_test_wav(AUDIO_PARAMETERS.max_sample_length_ms, 44100); // 1 second, 44100 Hz sample rate

        // Create a sample voice node
        let voice_node = VoiceNodeIngress {
            x: 0.0,
            y: 0.0,
            sample: test_wav,
        };

        // Add the voice node to the canister
        let add_result = pic
            .update_call(
                canister_id,
                Principal::anonymous(),
                "add_voice_node",
                candid::encode_one(voice_node).unwrap(),
            )
            .expect("Failed to call add_voice_node");

        match add_result {
            WasmResult::Reply(_) => {
                println!("Voice node successfully added.");
            }
            WasmResult::Reject(err) => {
                println!("add_voice_node rejected: {:?}", err);
                assert!(false);
            }
        }

        // Generate an angle file by querying the canister
        let angle = 0.0; // Using angle 0.0 for this test
        let get_angle_file_result = pic
            .query_call(
                canister_id,
                Principal::anonymous(),
                "get_angle_file",
                candid::encode_one(angle).unwrap(),
            )
            .expect("Failed to call get_angle_file");

        match get_angle_file_result {
            WasmResult::Reply(bytes) => {
                // Decode the response as an HttpStreamingResponse
                let decoded_response = Decode!(&bytes, HttpStreamingResponse);

                match decoded_response {
                    Ok(response) => {
                        println!("Headers: {:?}", response.headers);
                        println!("Body size: {:?}", response.body.len());

                        // Assert that the file was generated and headers exist
                        assert!(response.body.len() > 0);
                        assert!(response.headers.iter().any(|(k, _)| k == "content-type"));
                    }
                    Err(e) => {
                        println!("Failed to decode response: {:?}", e);
                        assert!(false);
                    }
                }
            }
            WasmResult::Reject(err) => {
                println!("get_angle_file rejected: {:?}", err);
                assert!(false);
            }
        }
    }

    #[test]
    fn pocket_ic_smoke_test_360_files() {
        let pic = PocketIc::new();
        let canister_id = pic_initialize_canister(&pic);

        // Loop through 360 degrees and add a voice node for each angle
        for angle in 0..360 {
            // Generate a test WAV audio file
            let test_wav = generate_test_wav(AUDIO_PARAMETERS.max_sample_length_ms, 44100);

            // Create a sample voice node, use angle to vary x and y
            let voice_node = VoiceNodeIngress {
                x: (angle as f64).cos() * (SIMULATION_PARAMETERS.logical_radius / 2.), // Vary x based on angle
                y: (angle as f64).sin() * (SIMULATION_PARAMETERS.logical_radius / 2.), // Vary y based on angle
                sample: test_wav,
            };

            // Add the voice node to the canister
            let add_result = pic
                .update_call(
                    canister_id,
                    Principal::anonymous(),
                    "add_voice_node",
                    candid::encode_one(voice_node).unwrap(),
                )
                .expect(&format!("Failed to call add_voice_node for angle {angle}"));

            match add_result {
                WasmResult::Reply(_) => {
                    println!("Voice node successfully added for angle {angle}.");
                }
                WasmResult::Reject(err) => {
                    println!("add_voice_node rejected for angle {angle}: {:?}", err);
                    assert!(false);
                }
            }
        }

        // Generate an angle file by querying the canister for an angle (e.g., 180 degrees)
        let test_angle = 180.0; // Using angle 180.0 for this test
        let get_angle_file_result = pic
            .query_call(
                canister_id,
                Principal::anonymous(),
                "get_angle_file",
                candid::encode_one(test_angle).unwrap(),
            )
            .expect("Failed to call get_angle_file");

        match get_angle_file_result {
            WasmResult::Reply(bytes) => {
                // Decode the response as an HttpStreamingResponse
                let decoded_response = Decode!(&bytes, HttpStreamingResponse);

                match decoded_response {
                    Ok(response) => {
                        println!("Headers: {:?}", response.headers);
                        println!("Body size: {:?}", response.body.len());

                        // Assert that the file was generated and headers exist
                        assert!(response.body.len() > 0);
                        assert!(response.headers.iter().any(|(k, _)| k == "content-type"));
                        assert!(false);
                    }
                    Err(e) => {
                        println!("Failed to decode response: {:?}", e);
                        assert!(false);
                    }
                }
            }
            WasmResult::Reject(err) => {
                println!("get_angle_file rejected: {:?}", err);
                assert!(false);
            }
        }
    }
}
