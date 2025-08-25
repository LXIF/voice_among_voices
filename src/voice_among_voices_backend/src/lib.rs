pub mod audio;
mod evm;
mod physics;
pub mod storage;
mod structs;
pub mod test_functions;
mod utils;
use audio::*;
mod http;
mod images;
use candid::Principal;
use ic_cdk::{
    api::{caller, performance_counter, set_certified_data, time},
    export_candid, init, post_upgrade, query, trap, update,
};
use ic_cdk_timers::set_timer;
use ic_http_certification::{utils::skip_certification_certified_data, HttpRequest, HttpResponse};
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
use utils::node_within_circle;
use voice_nodes::get_stored_voice_nodes;

use evm::{check_auth_for_single_node_id, get_caller_wallet_address, StorableAddress};

use crate::storage::{
    files_and_voices::get_file_for_angle_multicall,
    voice_nodes::update_stored_voice_node_without_simulation,
};

#[init]
fn init(maybe_arg: Option<VoiceAmongVoicesInit>) {
    initialize_storage(maybe_arg);
    set_certified_data(&skip_certification_certified_data());
}

#[query]
fn get_siwe_principal() -> Principal {
    siwe_principal()
}

#[post_upgrade]
fn post_upgrade(maybe_arg: Option<VoiceAmongVoicesInit>) {
    upgrade_storage(maybe_arg);
    set_certified_data(&skip_certification_certified_data());
}

#[update]
async fn update_voice_node(
    node: VoiceNodeIngress,
) -> Result<VoiceNodeEgressStore, AddVoiceNodeError> {
    match check_auth_for_single_node_id(node.id).await {
        Ok(address) => {
            let res = update_stored_voice_node(node, address, time());
            invalidate_angle_file_cache();
            zero_cache_update();
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
async fn get_angle_file(angle: u64, multicall: bool) -> HttpStreamingResponse {
    if angle > 360 || angle < 1 {
        ic_cdk::trap("invalid angle")
    };
    match check_auth_for_single_node_id(angle as usize).await {
        Ok(_address) => match multicall {
            false => get_file_for_angle(angle),
            true => match get_file_for_angle_multicall(angle) {
                MulticallResponse::HttpStreamingResponse(response) => response,
                MulticallResponse::Continue => {
                    let (result,) =
                        ic_cdk::call(ic_cdk::id(), "generate_file_for_angle_multicall", (angle,))
                            .await
                            .expect("Failed to generate file in get angle file multicall");
                    result
                }
                MulticallResponse::ZeroFinished => {
                    panic!("This should not happen.");
                }
            },
        },
        Err(err) => trap(&format!("{:?}", err)), //TODO: maybe use Result here
    }
}

#[update]
fn generate_file_for_angle(angle: u64) -> HttpStreamingResponse {
    if caller() != ic_cdk::id() {
        ic_cdk::trap("Unauthorized");
    }
    get_file_for_angle(angle)
}

#[update]
async fn generate_file_for_angle_multicall(angle: u64) -> HttpStreamingResponse {
    if caller() != ic_cdk::id() {
        ic_cdk::trap("Unauthorized");
    }

    match get_file_for_angle_multicall(angle) {
        MulticallResponse::HttpStreamingResponse(response) => response,
        MulticallResponse::Continue => {
            let (result,) =
                ic_cdk::call(ic_cdk::id(), "generate_file_for_angle_multicall", (angle,))
                    .await
                    .expect("Failed to generate file in generate file for angle multicall get file for angle multicall multicall response continue");
            result
        }
        MulticallResponse::ZeroFinished => HttpStreamingResponse {
            status_code: 200,
            headers: vec![("ZeroFinished".to_string(), "true".to_string())],
            body: ByteBuf::new(),
            upgrade: None,
            streaming_strategy: None,
        },
    }
}

#[update]
async fn populate_with_demo_content(from: u64, step: u64) -> Result<(), AuthorizationError> {
    if from + step >= 359 {
        return Ok(());
    }
    match check_auth_for_single_node_id(0).await {
        Ok(_) => {
            let sample = generate_test_wav(
                AUDIO_PARAMETERS.max_sample_length_ms,
                AUDIO_PARAMETERS.sample_rate,
                0.05,
            );

            for i in from..from + step {
                let node: VoiceNodeIngress = VoiceNodeIngress {
                    id: i as usize,
                    x: (i as f64).cos()
                        * (SIMULATION_PARAMETERS.logical_radius * (0.5 - (0.001 * i as f64))),
                    y: (i as f64).sin()
                        * (SIMULATION_PARAMETERS.logical_radius * (0.5 - (0.001 * i as f64))),
                    sample: sample.clone(),
                };
                update_stored_voice_node_without_simulation(
                    node,
                    alloy::primitives::Address::ZERO,
                    time(),
                )
                .expect(&format!("Failed to add voice node {i}"));
            }
            let _ = ic_cdk::call::<(u64, u64), ()>(
                ic_cdk::id(),
                "populate_with_demo_content",
                (from + step, step),
            )
            .await
            .expect("Failed to call populate_with_demo_content");
            Ok(())
        }
        Err(e) => Err(e),
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
    format!("{}", caller())
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
    match check_auth_for_single_node_id(admin_id() as usize).await {
        Ok(address) => {
            let res = files_and_voices::censor_voice(node_id, address, time());
            zero_cache_update();
            res
        }
        Err(err) => Err(err.into()),
    }
}

// ADVERTISEMENT
#[query]
fn get_token_buy_link() -> String {
    token_buy_link()
}

// CONFIG
#[query]
fn get_config() -> StorableConfig {
    storage::config()
}

// IMAGE GENERATION
#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    http::http_request(req)
}

export_candid!();
