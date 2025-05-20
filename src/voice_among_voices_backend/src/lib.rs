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
    api::{msg_caller, performance_counter, time},
    export_candid, init, post_upgrade, query, trap, update,
};
use ic_cdk_timers::set_timer;
use physics::*;
use serde_bytes::ByteBuf;
use std::{time::Duration, u64};
use storage::{
    files_and_voices::{get_file_for_angle, get_file_for_zero_angle, get_streaming_chunk},
    init::*,
    voice_log::VoiceLog,
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
fn post_upgrade(maybe_arg: Option<VoiceAmongVoicesInit>) {
    upgrade_storage(maybe_arg);
}

#[update]
async fn update_voice_node(
    node: VoiceNodeIngress,
) -> Result<VoiceNodeEgressStore, AddVoiceNodeError> {
    match check_auth_for_single_node_id(node.id).await {
        Ok(address) => {
            let res = update_stored_voice_node(node, address, time());
            let _ = ic_cdk_timers::set_timer(Duration::from_nanos(1), zero_cache_update); // TODO: use this result
            res
        }
        Err(err) => Err(err.into()),
    }
}

#[query]
fn get_voice_nodes() -> Result<VoiceNodeEgressStore, String> {
    Ok(get_stored_voice_nodes())
}

#[update]
async fn get_angle_file(angle: u64) -> HttpStreamingResponse {
    if angle > 360 || angle < 1 {
        ic_cdk::trap("invalid angle")
    };
    match check_auth_for_single_node_id(angle as usize).await {
        Ok(_address) => get_file_for_angle(angle),
        Err(err) => trap(format!("{:?}", err)), //TODO: maybe use Result here
    }
}

#[query]
fn get_zero_file() -> HttpStreamingResponse {
    get_file_for_zero_angle()
}

// #[update]
// fn update_zero_cache() {
//     zero_cache_update();
// }

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

// LOGS
#[query]
async fn get_voice_logs(skip: u64, take: u64) -> Vec<VoiceLog> {
    retrieve_voice_logs(skip as usize, take as usize)
}

// CENSORSHIP
#[query]
fn get_admin_id() -> u64 {
    admin_id()
}

#[query]
async fn get_voice(node_id: u64) -> Result<AudioSample, CensorshipError> {
    match check_auth_for_single_node_id(admin_id() as usize).await {
        Ok(_) => files_and_voices::get_voice(node_id).ok_or(CensorshipError::VoiceNotFound),
        Err(err) => Err(err.into()),
    }
}

#[update]
async fn censor(node_id: u64) -> Result<(), CensorshipError> {
    match check_auth_for_single_node_id(0).await {
        Ok(address) => files_and_voices::censor_voice(node_id, address, time()),
        Err(err) => Err(err.into()),
    }
}

export_candid!();
