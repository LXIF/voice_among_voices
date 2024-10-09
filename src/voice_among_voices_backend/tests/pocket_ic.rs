use candid::{define_function, CandidType, Decode, Deserialize, Principal};
use hound::{WavSpec, WavWriter};
use ic_http_certification::HeaderField;
use pocket_ic::{PocketIc, WasmResult};
use serde_bytes::ByteBuf;
use std::io::Cursor;
use std::{fs::read, path::PathBuf};

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
        let test_wav = generate_test_wav(AUDIO_PARAMETERS.max_sample_length_ms / 20, 44100);

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

#[derive(Debug, Clone, Copy, CandidType)]
struct AudioParameters {
    total_length_ms: u32,
    max_sample_length_ms: u32,
    sample_rate: u32,
    chunk_size: usize,
    fade_ms: u32,
}

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

#[derive(Debug, Clone, Copy, CandidType, Deserialize)]
pub struct SimulationParameters {
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

fn generate_test_wav(duration_ms: u32, sample_rate: u32) -> Vec<u8> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buffer = Vec::new();
    let mut writer = WavWriter::new(Cursor::new(&mut buffer), spec).unwrap();

    let num_samples = (sample_rate * duration_ms / 1000) as usize;
    let amplitude = i16::MAX as f32;

    for t in 0..num_samples {
        let sample = ((t as f32 / sample_rate as f32) * 440. * 2. * std::f32::consts::PI).sin();
        writer.write_sample((sample * amplitude) as i16).unwrap();
    }

    writer.finalize().unwrap();

    buffer
}

define_function!(pub CallbackFunc : (StreamingCallbackToken) -> (StreamingCallbackHttpResponse) query);

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StreamingCallbackHttpResponse {
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub token: Option<StreamingCallbackToken>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StreamingCallbackToken {
    pub angle: u32,
    pub chunk_index: u32,
    pub chunks: u32,
    pub auth_token: Option<ByteBuf>,
}

#[derive(CandidType, Deserialize, Clone)]
pub enum StreamingStrategy {
    Callback {
        token: StreamingCallbackToken,
        callback: CallbackFunc,
    },
}

#[derive(CandidType, Deserialize, Clone, Default)]
pub struct HttpStreamingResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub upgrade: Option<bool>,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(Clone, Debug, Deserialize, CandidType)]
struct VoiceNodeIngress {
    x: f64,
    y: f64,
    sample: Vec<u8>, // here it's still a blob
}

fn pic_initialize_canister(pic: &PocketIc) -> Principal {
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);

    let mut wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    wasm_path.pop();
    wasm_path.pop();
    wasm_path
        .push(".dfx/local/canisters/voice_among_voices_backend/voice_among_voices_backend.wasm");
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
