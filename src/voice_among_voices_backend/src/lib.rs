pub mod audio;
mod evm;
mod physics;
pub mod storage;
mod structs;
pub mod test_functions;
mod utils;

use audio::*;
use candid::Principal;
use ic_cdk::{
    api::{msg_caller, performance_counter},
    export_candid, init, post_upgrade, query, update,
};
use ic_cdk_timers::set_timer;
use physics::*;
use serde_bytes::ByteBuf;
use std::{time::Duration, u64};
use storage::{
    files_and_voices::{get_file_for_angle, get_file_for_zero_angle, get_streaming_chunk},
    init::*,
    voice_nodes::update_stored_voice_node,
    *,
};
use structs::*;
use test_functions::generate_test_wav;
use utils::{node_within_circle, split_into_chunks};
use voice_nodes::get_stored_voice_nodes;

use evm::{check_auth_for_single_node_id, get_caller_wallet_address, StorableAddress};

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
    check_auth_for_single_node_id(node.id).await; // traps if not authorized
    update_stored_voice_node(node)
}

#[query]
fn get_voice_nodes() -> VoiceNodeEgressStore {
    get_stored_voice_nodes()
}

#[update]
async fn get_angle_file(angle: u64) -> HttpStreamingResponse {
    if angle > 360 || angle < 1 {
        ic_cdk::trap("invalid angle")
    };
    check_auth_for_single_node_id(angle as usize).await;
    get_file_for_angle(angle)
}

#[query]
fn get_zero_file() -> HttpStreamingResponse {
    get_file_for_zero_angle()
}

#[update]
fn update_zero_cache() {
    zero_cache_update();
}

#[query]
fn http_request_streaming_callback(token: StreamingCallbackToken) -> StreamingCallbackHttpResponse {
    get_streaming_chunk(token)
}

#[query]
fn get_my_principal() -> String {
    format!("{}", msg_caller())
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
#[query]
async fn get_token_address() -> AddressEgress {
    AddressEgress::from(token_address())
}

#[query(composite = true)]
async fn get_wallet_address() -> Result<String, String> {
    get_caller_wallet_address().await
}

export_candid!();
