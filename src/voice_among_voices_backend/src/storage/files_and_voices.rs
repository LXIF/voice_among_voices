use alloy::primitives::Address;

use crate::{
    audio::{generate_or_add_angle_vectors, vectors_to_maybe_file},
    generate_angle_file, performance_counter, set_timer, split_into_chunks,
    storage::ANGLE_FILE_WIP_CACHE,
    structs::{AudioSample, CensorshipError, MulticallResponse, StorableAudioSample},
    test_functions::{generate_test_sample_vec, generate_test_wav},
    ByteBuf, Duration, HttpStreamingResponse, StreamingCallbackHttpResponse,
    StreamingCallbackToken, StreamingStrategy, ANGLE_FILE_EGRESS_CACHE, AUDIO_PARAMETERS,
    SAMPLES_MEMORY, SIMULATION_PARAMETERS, STREAMING_CALLBACK, VOICE_NODES_MEMORY,
    ZERO_DEGREE_FILE_CACHE,
};

use super::{
    store_voice_log,
    voice_log::{VoiceAction, VoiceLog},
};

pub fn get_file_for_angle(angle: u64) -> HttpStreamingResponse {
    let beginning_cost = performance_counter(0);

    let mut result: Vec<u8> = vec![];

    VOICE_NODES_MEMORY.with_borrow(|nodes| {
        SAMPLES_MEMORY.with_borrow(|samples_map| {
            result = generate_angle_file(
                angle as f64,
                nodes,
                samples_map,
                &AUDIO_PARAMETERS,
                &SIMULATION_PARAMETERS,
            )
            .unwrap(); // TODO: error handling
        });
    });

    let (token, first_chunk) = prepare_stream_file(result, angle);

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

fn prepare_stream_file(file: Vec<u8>, angle: u64) -> (StreamingCallbackToken, Vec<u8>) {
    let chunks = split_into_chunks(file, &AUDIO_PARAMETERS);

    ANGLE_FILE_EGRESS_CACHE.with_borrow_mut(|cache| {
        cache.insert(angle as u32, chunks.clone());
    });

    set_timer(Duration::from_secs(180), move || {
        // invalidate after 180s for now
        ANGLE_FILE_EGRESS_CACHE.with_borrow_mut(|cache| {
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

    (token, first_chunk)
}

fn set_zero_file(new_file: Vec<u8>) {
    ZERO_DEGREE_FILE_CACHE.with_borrow_mut(|cache| {
        SAMPLES_MEMORY.with_borrow(|samples| {
            VOICE_NODES_MEMORY.with_borrow(|nodes| {
                let chunks = split_into_chunks(new_file, &AUDIO_PARAMETERS);
                cache.clear();
                cache.extend(chunks.clone());
            })
        });
    });
}

pub fn get_file_for_angle_multicall(angle: u64) -> MulticallResponse {
    ANGLE_FILE_WIP_CACHE.with_borrow_mut(|wip_cache| {
        VOICE_NODES_MEMORY.with_borrow(|nodes| {
            SAMPLES_MEMORY.with_borrow(|samples| {
                // check if wip exists
                let wip = wip_cache.remove(&angle);
                // do the thing
                let new_wip = generate_or_add_angle_vectors(
                    angle,
                    nodes,
                    samples,
                    &AUDIO_PARAMETERS,
                    &SIMULATION_PARAMETERS,
                    wip,
                );

                // if finished, return file
                match vectors_to_maybe_file(&new_wip) {
                    Some(maybe_file_result) => {
                        let result = maybe_file_result.unwrap();

                        if angle == 0 {
                            set_zero_file(result);
                            return MulticallResponse::ZeroFinished;
                        }

                        let (token, first_chunk) = prepare_stream_file(result, angle);

                        MulticallResponse::HttpStreamingResponse(HttpStreamingResponse {
                            status_code: 200,
                            headers: vec![
                                ("content-type".to_string(), "audio/wav".to_string()),
                                ("x-n-calls".to_string(), new_wip.n_calls.to_string()), // Profiling header
                            ],
                            body: ByteBuf::from(first_chunk),
                            upgrade: None,
                            streaming_strategy: create_strategy(token),
                        })
                    }
                    None => {
                        wip_cache.insert(angle, new_wip);
                        MulticallResponse::Continue
                    }
                }
            })
        })
    })
}

pub fn get_file_for_zero_angle() -> HttpStreamingResponse {
    ZERO_DEGREE_FILE_CACHE.with_borrow(|chunks| {
        let total_chunks = chunks.len() as u32;

        let first_chunk = chunks.get(0).cloned().unwrap_or_default();
        let token = StreamingCallbackToken {
            angle: 0 as u32,
            chunk_index: 0,
            chunks: total_chunks,
            auth_token: None, // TODO: maybe implement this for security purposes
        };

        HttpStreamingResponse {
            status_code: 200,
            headers: vec![("content-type".to_string(), "audio/wav".to_string())],
            body: ByteBuf::from(first_chunk),
            upgrade: None,
            streaming_strategy: create_strategy(token),
        }
    })
}

pub fn get_streaming_chunk(token: StreamingCallbackToken) -> StreamingCallbackHttpResponse {
    let chunks: Vec<Vec<u8>>;

    match token.angle {
        0 => {
            chunks = ZERO_DEGREE_FILE_CACHE.with_borrow(|cache| cache.clone());
        }
        _ => {
            chunks = match ANGLE_FILE_EGRESS_CACHE
                .with_borrow(|cache| cache.get(&token.angle).cloned())
            {
                Some(file_chunks) => file_chunks,
                None => ic_cdk::trap("Cache out of date, connection too slow"),
            };
        }
    }

    if let Some(token) = token.next() {
        if let Some(chunk) = chunks.get((token.chunk_index) as usize) {
            StreamingCallbackHttpResponse {
                headers: vec![],
                body: ByteBuf::from(chunk.clone()),
                token: Some(token),
            }
        } else {
            ic_cdk::trap("Chunk not found");
        }
    } else {
        StreamingCallbackHttpResponse {
            headers: vec![],
            body: ByteBuf::new(),
            token: None,
        }
    }
}

pub fn get_voice(node_id: u64) -> Option<AudioSample> {
    SAMPLES_MEMORY
        .with_borrow(|samples| samples.get(node_id))
        .and_then(|sample| Some(sample.to_wav()))
}

pub fn censor_voice(node_id: u64, address: Address, timestamp: u64) -> Result<(), CensorshipError> {
    SAMPLES_MEMORY.with_borrow_mut(|samples| {
        let sample = samples.get(node_id).unwrap(); // TODO improve
        let length_samples = sample.sample_length_samples;
        let censored_sample = generate_censored_sample_vec(
            length_samples,
            crate::storage::AUDIO_PARAMETERS.sample_rate,
        );
        samples.set(
            node_id,
            &StorableAudioSample {
                sample: censored_sample,
                ..sample
            },
        );
    });
    store_voice_log(VoiceLog::new(
        timestamp,
        node_id,
        VoiceAction::Censor,
        address.into(),
        None,
    ))
    .map_err(|err| CensorshipError::InternalCanisterError(format!("{}", err)))
}

fn create_strategy(token: StreamingCallbackToken) -> Option<StreamingStrategy> {
    let callback = STREAMING_CALLBACK.clone();

    Some(StreamingStrategy::Callback { token, callback })
}

fn generate_censored_wav(duration_samples: u32, sample_rate: u32) -> Vec<u8> {
    let duration_ms = duration_samples / sample_rate * 1000;

    generate_test_wav(duration_ms, sample_rate, 0.1)
}

fn generate_censored_sample_vec(duration_samples: u32, sample_rate: u32) -> Vec<i16> {
    let duration_ms = duration_samples / sample_rate * 1000;

    generate_test_sample_vec(duration_ms, sample_rate, 0.1)
}
