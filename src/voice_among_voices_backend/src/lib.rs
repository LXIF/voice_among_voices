pub mod audio;
mod evm;
mod physics;
pub mod storage;
mod structs;
pub mod test_functions;
mod utils;

use alloy::primitives::Address;
use audio::*;
use candid::{CandidType, Principal};
use ic_cdk::{
    api::{caller, performance_counter},
    export_candid, init, post_upgrade, query, update,
};
use ic_cdk_timers::set_timer;
use ic_stable_structures::{
    cell::ValueError,
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl, StableCell, StableVec,
};
use once_cell::sync::Lazy;
use physics::*;
use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::{cell::RefCell, collections::HashMap, str::FromStr, time::Duration, u64};
use storage::{init::*, voice_nodes::update_stored_voice_node, *};
use structs::*;
use test_functions::generate_test_wav;
use utils::{node_within_circle, split_into_chunks};
use voice_nodes::get_stored_voice_nodes;

use evm::{
    caller_is_owner_of, check_auth_for_single_node, get_caller_balance, get_caller_owned_tokens,
    get_caller_wallet_address, StorableAddress,
};

#[init]
fn init(maybe_arg: Option<VoiceAmongVoicesInit>) {
    initialize_storage(maybe_arg);
}

#[query]
fn get_siwe_principal() -> Principal {
    siwe_principal()
}

#[post_upgrade]
fn post_upgrade() {
    collider_init();
}

#[update]
async fn update_voice_node(
    node: VoiceNodeIngress,
) -> Result<VoiceNodeEgressStore, AddVoiceNodeError> {
    check_auth_for_single_node(&node).await; // traps if not authorized
    update_stored_voice_node(node)
}

#[query]
fn get_voice_nodes() -> VoiceNodeEgressStore {
    get_stored_voice_nodes()
}

#[query]
fn get_my_voice() -> Option<AudioSample> {
    //TODO: guard and return 'owned' sample
    let mut result: Option<AudioSample> = None;
    let index = 0;

    VOICE_NODES_MEMORY.with_borrow(|nodes| {
        SAMPLES_MEMORY.with_borrow(|samples| {
            if let Some(node) = nodes.get(index) {
                if node.sample_id != u64::MAX {
                    if let Some(sample) = samples.get(node.sample_id) {
                        result = Some(sample);
                    }
                }
            }
        });
    });

    result
}

#[update]
fn get_angle_file(angle: f64) -> HttpStreamingResponse {
    // TODO: guard to user, restrict angle
    let beginning_cost = performance_counter(0);

    let mut result: Vec<u8> = vec![];

    VOICE_NODES_MEMORY.with_borrow(|nodes| {
        SAMPLES_MEMORY.with_borrow(|samples_map| {
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

#[query]
fn get_zero_file() -> HttpStreamingResponse {
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

#[query]
fn http_request_streaming_callback(token: StreamingCallbackToken) -> StreamingCallbackHttpResponse {
    let chunks: Vec<Vec<u8>>;

    match token.angle {
        0 => {
            chunks = ZERO_DEGREE_FILE_CACHE.with_borrow(|cache| cache.clone());
        }
        _ => {
            chunks = match ANGLE_FILE_CACHE.with_borrow(|cache| cache.get(&token.angle).cloned()) {
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

#[query]
fn get_my_principal() -> String {
    format!("{}", caller())
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
    get_stored_audio_parameters()
}

// EVM
#[query(composite = true)]
async fn get_wallet_address() -> Result<String, String> {
    get_caller_wallet_address().await
}

#[update]
async fn get_owned_tokens() -> Result<Vec<String>, String> {
    get_caller_owned_tokens()
        .await
        .map(|tokens| tokens.into_iter().map(|t| t.to_string()).collect())
}

#[update]
async fn get_balance() -> Result<String, String> {
    get_caller_balance().await.map(|bal| bal.to_string())
}

#[update]
async fn is_owner_of(token_id: u64) -> Result<bool, String> {
    caller_is_owner_of(token_id).await
}

export_candid!();
