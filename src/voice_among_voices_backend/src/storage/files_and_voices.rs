use alloy::primitives::Address;
use ic_cdk_timers::{clear_timer, TimerId};

use crate::{
    audio::{generate_or_add_angle_vectors, vectors_to_maybe_file},
    generate_angle_file_chunks, performance_counter, set_timer,
    storage::{
        cache_angle_file_chunks, get_maybe_cached_angle_file, get_maybe_cached_angle_file_epoch,
        ANGLE_FILE_EGRESS_TIMERS, ANGLE_FILE_WIP_CACHE, VOICE_LOG,
    },
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

const CHUNK_CACHE_TIMEOUT_SECONDS: u64 = 300;

pub fn get_file_for_angle(angle: u64) -> HttpStreamingResponse {
    let beginning_cost = performance_counter(0);

    let result = get_maybe_cached_angle_file(angle).unwrap_or_else(|| {
        VOICE_NODES_MEMORY.with_borrow(|nodes| {
            SAMPLES_MEMORY.with_borrow(|samples_map| {
                let file = generate_angle_file_chunks(
                    angle as f64,
                    nodes,
                    samples_map,
                    &AUDIO_PARAMETERS,
                    &SIMULATION_PARAMETERS,
                )
                .unwrap(); // TODO: error handling
                cache_angle_file_chunks(angle, file.clone());
                file
            })
        })
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

fn prepare_stream_file(chunk_files: Vec<Vec<u8>>, angle: u64) -> (StreamingCallbackToken, Vec<u8>) {
    // Clear old timers if still around
    with_egress_timers(|timers| {
        if let Some(old_timer_id) = timers.get(&angle) {
            clear_timer(*old_timer_id);
        }
    });

    let timer_id = set_timer(
        Duration::from_secs(CHUNK_CACHE_TIMEOUT_SECONDS),
        move || {
            // invalidate after 360s for now
            with_egress_cache_mut(|cache| {
                with_egress_timers_mut(|timers| {
                    cache.remove(&angle);
                    timers.remove(&angle);
                });
            });
        },
    );

    with_egress_cache_mut(|cache| {
        with_egress_timers_mut(|timers| {
            cache.insert(angle, chunk_files.clone());
            timers.insert(angle, timer_id);
        });
    });

    let total_chunks = chunk_files.len() as u32;

    let first_chunk = chunk_files.get(0).cloned().unwrap_or_default();
    let token = StreamingCallbackToken {
        angle: angle as u32,
        chunk_index: 0,
        chunks: total_chunks,
        auth_token: None, // TODO: maybe implement this for security purposes
    };

    (token, first_chunk)
}

fn try_get_chunk_response_and_refresh_timers(
    angle: u64,
) -> Option<(StreamingCallbackToken, Vec<u8>)> {
    with_egress_cache_mut(|cache| {
        if let Some(chunks) = cache.get(&angle) {
            // Refresh timer
            with_egress_timers_mut(|timers| {
                let old_timer_id = timers
                    .get(&angle)
                    .expect("attempting to refresh nonexistent timer");
                clear_timer(*old_timer_id);
                let fresh_timer_id = set_timer(Duration::from_secs(60 * 60 * 24 * 3), move || {
                    with_egress_cache_mut(|cache| {
                        with_egress_timers_mut(|timers| {
                            timers.remove(&angle);
                            cache.remove(&angle);
                        });
                    });
                });
                timers.insert(angle, fresh_timer_id);
            });
            let total_chunks = chunks.len() as u32;

            let first_chunk = chunks.get(0).cloned().unwrap_or_default();
            let token = StreamingCallbackToken {
                angle: angle as u32,
                chunk_index: 0,
                chunks: total_chunks,
                auth_token: None, // TODO: maybe implement this for security purposes
            };

            Some((token, first_chunk))
        } else {
            None
        }
    })
}

fn set_zero_file(new_file_chunks: Vec<Vec<u8>>) {
    ZERO_DEGREE_FILE_CACHE.with_borrow_mut(|cache| {
        cache.clear();
        cache.extend(new_file_chunks.clone());
    })
}

pub fn get_file_for_angle_multicall(angle: u64) -> MulticallResponse {
    // FIRST WE CHECK WHETHER WE HAVE A VALID CHUNK CACHE
    let current_epoch = VOICE_LOG.with_borrow(|log| log.len());
    let maybe_cached_epoch = get_maybe_cached_angle_file_epoch(angle);
    if let Some(cached_epoch) = maybe_cached_epoch {
        // IF THE CURRENT EPOCH EQUALS THAT OF THE CACHED ANGLE FILE, ANY CHUNK CACHES ARE OF THAT AS WELL
        if current_epoch == cached_epoch {
            // IF WE HAVE A CACHED CHUNK RESPONSE WE RETURN THAT
            if let Some((token, first_chunk)) = try_get_chunk_response_and_refresh_timers(angle) {
                return MulticallResponse::HttpStreamingResponse(HttpStreamingResponse {
                    status_code: 200,
                    headers: vec![
                        ("content-type".to_string(), "audio/wav".to_string()),
                        ("x-from".to_string(), "cached".to_string()),
                    ],
                    body: ByteBuf::from(first_chunk),
                    upgrade: None,
                    streaming_strategy: create_strategy(token),
                });
            } else {
                // OTHERWISE WE REPOPULATE THE CHUNK CACHE
                if let Some(file_chunks) = get_maybe_cached_angle_file(angle) {
                    let (token, first_chunk) = prepare_stream_file(file_chunks, angle);

                    return MulticallResponse::HttpStreamingResponse(HttpStreamingResponse {
                        status_code: 200,
                        headers: vec![
                            ("content-type".to_string(), "audio/wav".to_string()),
                            ("x-from".to_string(), "repopulated".to_string()),
                        ],
                        body: ByteBuf::from(first_chunk),
                        upgrade: None,
                        streaming_strategy: create_strategy(token),
                    });
                }
            }
        }
    }

    // IF THERE IS NO CACHED EPOCH, WE GENERATE

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

                        cache_angle_file_chunks(angle, result.clone());

                        let (token, first_chunk) = prepare_stream_file(result, angle);

                        MulticallResponse::HttpStreamingResponse(HttpStreamingResponse {
                            status_code: 200,
                            headers: vec![
                                ("content-type".to_string(), "audio/wav".to_string()),
                                ("x-n-calls".to_string(), new_wip.n_calls.to_string()), // Profiling header
                                ("x-from".to_string(), "fresh".to_string()),
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
    let angle = token.angle as u64;

    match token.angle {
        0 => {
            chunks = ZERO_DEGREE_FILE_CACHE.with_borrow(|cache| cache.clone());
        }
        _ => {
            chunks = match with_egress_cache(|cache| cache.get(&angle).cloned()) {
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
        with_egress_cache_mut(|cache| cache.remove(&angle));
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

fn with_egress_cache<F, R>(f: F) -> R
where
    F: FnOnce(&std::collections::HashMap<u64, Vec<Vec<u8>>>) -> R,
{
    ANGLE_FILE_EGRESS_CACHE.with_borrow(f)
}

fn with_egress_cache_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut std::collections::HashMap<u64, Vec<Vec<u8>>>) -> R,
{
    ANGLE_FILE_EGRESS_CACHE.with_borrow_mut(f)
}

fn with_egress_timers<F, R>(f: F) -> R
where
    F: FnOnce(&std::collections::HashMap<u64, TimerId>) -> R,
{
    ANGLE_FILE_EGRESS_TIMERS.with_borrow(f)
}

fn with_egress_timers_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut std::collections::HashMap<u64, TimerId>) -> R,
{
    ANGLE_FILE_EGRESS_TIMERS.with_borrow_mut(f)
}
