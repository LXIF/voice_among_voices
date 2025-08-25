use crate::audio::*;
use crate::physics::*;
use crate::storage::files_and_voices::get_file_for_angle_multicall;
use crate::structs::*;
use crate::utils::split_into_chunks;
use crate::StorableAddress;
use alloy::primitives::Address;
use candid::CandidType;
use candid::Principal;

use crate::storage::voice_log::VoiceLog;
use ic_cdk::api::caller;
use ic_stable_structures::{
    cell::ValueError,
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl, StableCell, StableVec,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use std::{cell::RefCell, collections::HashMap};

pub mod files_and_voices;
pub mod init;
pub mod voice_log;
pub mod voice_nodes;

thread_local! {
    // STABLE STORAGE
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static VOICE_NODES_MEMORY: RefCell<VoiceNodeLocalMemory> = RefCell::new(
        StableVec::init(MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(0)))).expect("Failed to initialize voice nodes map")
    );
    pub static SAMPLES_MEMORY: RefCell<AudioSampleMemory> = RefCell::new(
        StableVec::init(MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(1)))).expect("Failed to initialize samples map")
    );
    pub static SIWE_PRINCIPAL: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(2))), Principal::anonymous()).expect("Failed to initialize siwe principal storage")
    );
    pub static TOKEN_ADDRESS: RefCell<StableCell<StorableAddress, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(3))), StorableAddress(Address::ZERO)).expect("Failed to initialize token address storage")
    );
    pub static CONFIG: RefCell<StableCell<StorableConfig, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(4))), StorableConfig::default()).expect("Failed to initialize config storage")
    );
    pub static VOICE_LOG: RefCell<StableVec<VoiceLog, Memory>> = RefCell::new(
        StableVec::init(MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(5)))).expect("Failed to initialize log storage")
    );
    // STATE
    pub static COLLIDER_COORDINATES: RefCell<Vec<ColliderCoordinate>> = RefCell::new(vec![]);
    pub static FADES: RefCell<FadesCache> = RefCell::new(FadesCache { fade_in: vec![], fade_out: vec![] });
    pub static ANGLE_FILE_FILE_CACHE: RefCell<FileCache> = RefCell::new(HashMap::new());
    pub static ANGLE_FILE_EGRESS_CACHE: RefCell<ChunksCache> = RefCell::new(HashMap::new());
    pub static ANGLE_FILE_EGRESS_TIMERS: RefCell<CacheTimers> = RefCell::new(HashMap::new());
    pub static ANGLE_FILE_WIP_CACHE: RefCell<WipCache> = RefCell::new(HashMap::new());
    pub static ZERO_DEGREE_FILE_CACHE: RefCell<Vec<Channels>> = RefCell::new(vec![]);
}

pub static STREAMING_CALLBACK: Lazy<CallbackFunc> =
    Lazy::new(|| CallbackFunc::new(caller(), "http_request_streaming_callback".to_string()));

pub const AUDIO_PARAMETERS: AudioParameters = AudioParameters {
    total_length_ms: 4 * 60 * 1000,
    max_sample_length_ms: 10_000,
    sample_rate: 44100,
    chunk_size: 1024 * 1024,
    fade_ms: 30,
    n_process_per_call: 180,
};
pub const SIMULATION_PARAMETERS: SimulationParameters = SimulationParameters {
    velocity_cutoff: 0.2,
    force_cutoff: 100.,
    max_distance: 20.,
    force_strength: 3000.,
    linear_damping: 35.,
    logical_radius: 50.,
    n_collider_vertices: 90,
    friction: 0.7,
    density: 1.,
    max_steps: 500,
};

pub fn get_stored_audio_parameters() -> AudioParameters {
    AUDIO_PARAMETERS.clone()
}

pub fn zero_cache_update() {
    // ZERO_DEGREE_FILE_CACHE.with_borrow_mut(|cache| {
    //     SAMPLES_MEMORY.with_borrow(|samples| {
    //         VOICE_NODES_MEMORY.with_borrow(|nodes| {
    //             let new_file = generate_angle_file(
    //                 0 as f64,
    //                 nodes,
    //                 samples,
    //                 &AUDIO_PARAMETERS,
    //                 &SIMULATION_PARAMETERS,
    //             )
    //             .expect("failed to init zero cache");
    //             let chunks = split_into_chunks(new_file, &AUDIO_PARAMETERS);
    //             cache.clear();
    //             cache.extend(chunks.clone());
    //         })
    //     });
    // });
    let _ = ic_cdk_timers::set_timer(Duration::from_nanos(1), || {
        ic_cdk::spawn(zero_cache_update_multicall());
    });
}

async fn zero_cache_update_multicall() {
    match get_file_for_angle_multicall(0) {
        MulticallResponse::ZeroFinished => {
            ic_cdk::println!("Finished regenerating Zero file!");
            // Banger! We're done here UwU (Finished without outcall)
        }
        MulticallResponse::HttpStreamingResponse(_response) => {
            // This shouldn't happen
            ic_cdk::println!("Received wrong response for zero angle update!");
        }
        MulticallResponse::Continue => {
            let (result,) =
                ic_cdk::call(ic_cdk::id(), "generate_file_for_angle_multicall", (0u64,))
                    .await
                    .expect(
                        "Failed to generate file in zero cache update multicall response continue",
                    );

            match result {
                HttpStreamingResponse { headers, .. } => {
                    // Check if it has the ZeroFinished header
                    let is_zero_finished = headers
                        .iter()
                        .any(|header| header.0 == "ZeroFinished" && header.1 == "true");

                    if is_zero_finished {
                        // Great! We are finished.
                        ic_cdk::println!("Finished regenerating zero file!");
                    } else {
                        ic_cdk::println!("Got invalid zero cache update multicall response!");
                    }
                }
            }
        }
    }
}

pub fn store_siwe_principal(principal: Principal) -> Result<Principal, ValueError> {
    SIWE_PRINCIPAL.with_borrow_mut(|siwe_principal| siwe_principal.set(principal))
}

pub fn siwe_principal() -> Principal {
    SIWE_PRINCIPAL.with_borrow(|principal| principal.get().clone())
}

pub fn store_token_address(address: Address) -> Result<StorableAddress, ValueError> {
    TOKEN_ADDRESS.with_borrow_mut(|token_address| token_address.set(StorableAddress(address)))
}

pub fn token_address() -> Address {
    TOKEN_ADDRESS.with_borrow(|token_address| token_address.get().0)
}

pub fn store_voice_log(log: VoiceLog) -> Result<(), ic_stable_structures::GrowFailed> {
    VOICE_LOG.with_borrow_mut(|log_vec| log_vec.push(&log))
}

pub fn retrieve_voice_logs(skip: usize, take: usize) -> Vec<VoiceLog> {
    VOICE_LOG.with_borrow(|log_vec| log_vec.iter().rev().skip(skip).take(take).collect())
}

pub fn get_fades() -> FadesCache {
    FADES.with_borrow(|fades| fades.clone())
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct AddressEgress {
    pub address: String,
}

impl From<Address> for AddressEgress {
    fn from(addr: Address) -> Self {
        AddressEgress {
            address: format!("{:#x}", addr),
        }
    }
}

pub fn store_dev_mode(dev_mode: bool) -> Result<StorableConfig, ValueError> {
    CONFIG.with_borrow_mut(|config| {
        let mut current = config.get().clone();
        current.dev_mode = dev_mode;
        config.set(current)
    })
}

pub fn dev_mode() -> bool {
    CONFIG.with_borrow(|config| config.get().dev_mode)
}

pub fn store_admin_token_id(id: u64) -> Result<StorableConfig, ValueError> {
    CONFIG.with_borrow_mut(|config| {
        let mut current = config.get().clone();
        current.admin_id = id;
        config.set(current)
    })
}

pub fn store_token_buy_link(token_buy_link: String) -> Result<StorableConfig, ValueError> {
    CONFIG.with_borrow_mut(|config| {
        let mut current = config.get().clone();
        current.token_buy_link = token_buy_link;
        config.set(current)
    })
}

pub fn admin_id() -> u64 {
    CONFIG.with_borrow(|config| config.get().admin_id)
}

pub fn token_buy_link() -> String {
    CONFIG.with_borrow(|config| config.get().token_buy_link.clone())
}

pub fn config() -> StorableConfig {
    CONFIG.with_borrow(|config| config.get().clone())
}

pub fn get_maybe_cached_angle_file_epoch(angle: u64) -> Option<u64> {
    ANGLE_FILE_FILE_CACHE.with_borrow(|cache| {
        cache
            .get(&angle.try_into().expect("unexpected overflow"))
            .and_then(
                |CachedChannels {
                     left_channel: _,
                     right_channel: _,
                     epoch,
                 }| Some(epoch).copied(),
            )
    })
}

pub fn get_maybe_cached_angle_channels(angle: u64) -> Option<CachedChannels> {
    ANGLE_FILE_FILE_CACHE.with_borrow(|cache| {
        cache
            .get(&angle.try_into().expect("unexpected overflow"))
            .cloned()
            .and_then(|cached_chans| Some(cached_chans))
    })
}

pub fn cache_angle_file(angle: u64, left_channel: Vec<i16>, right_channel: Vec<i16>) -> u64 {
    VOICE_LOG.with_borrow(|log| {
        ANGLE_FILE_FILE_CACHE.with_borrow_mut(|cache| {
            let epoch = log.len();
            cache.insert(
                angle.try_into().expect("unexpected overflow"),
                CachedChannels {
                    left_channel,
                    right_channel,
                    epoch,
                },
            );

            epoch
        })
    })
}

pub fn invalidate_angle_file_cache() {
    ANGLE_FILE_FILE_CACHE.with_borrow_mut(|cache| cache.clear())
}

#[cfg(test)]
mod tests {
    use super::{init::*, voice_nodes::*, *};
    use crate::test_functions::*;

    #[test]
    fn voice_nodes_get_added_correctly() {
        collider_init();
        nodes_init();
        samples_init();

        let voice_node = VoiceNodeIngress {
            x: 0.,
            y: 40.,
            sample: generate_test_wav(1000, 44100, 1.),
            id: 1,
        };

        let another_voice_node = VoiceNodeIngress {
            x: -45.,
            y: 0.,
            sample: generate_test_wav(1000, 44100, 1.),
            id: 2,
        };

        let result_a = update_stored_voice_node(voice_node, Address::ZERO, 0u64);
        println!("{:#?}", result_a.unwrap());
        let _ = update_stored_voice_node(another_voice_node, Address::ZERO, 0u64);

        SAMPLES_MEMORY.with_borrow(|samples_map| {
            let id = samples_map.get(1).expect("No sample!").id;
            let another_id = samples_map.get(2).expect("No another_sample!").id;

            assert_eq!(id, 1);
            assert_eq!(another_id, 2);
        });
    }

    #[test]
    fn get_correct_audio_params() {
        let audio_params = get_stored_audio_parameters();

        assert!(audio_params.max_sample_length_ms > 0);
    }

    #[test]
    fn out_of_bounds_voice_nodes_get_rejected_correctly() {
        collider_init();

        let voice_node = VoiceNodeIngress {
            x: -50.,
            y: -50.,
            sample: generate_test_wav(1000, 44100, 1.),
            id: 0,
        };

        let another_voice_node = VoiceNodeIngress {
            x: 99.,
            y: 50.,
            sample: generate_test_wav(10000, 44100, 1.),
            id: 1,
        };

        let _ = update_stored_voice_node(voice_node, Address::ZERO, 0u64);
        let _ = update_stored_voice_node(another_voice_node, Address::ZERO, 0u64);

        SAMPLES_MEMORY.with_borrow(|samples_map| {
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
            sample: generate_test_wav(max_length + 10, 44100, 1.),
            id: 0,
        };

        let another_voice_node = VoiceNodeIngress {
            x: 60.,
            y: 60.,
            sample: generate_test_wav(max_length + 20, 44100, 1.),
            id: 1,
        };

        let _ = update_stored_voice_node(voice_node, Address::ZERO, 0u64);
        let _ = update_stored_voice_node(another_voice_node, Address::ZERO, 0u64);

        SAMPLES_MEMORY.with_borrow(|samples_map| {
            println!("{}", samples_map.len());
            assert!(samples_map.len() == 0);
        });
    }

    #[test]
    fn init_creates_coordinates() {
        initialize_storage(None);
        COLLIDER_COORDINATES.with_borrow(|collider_coordinates| {
            let n = SIMULATION_PARAMETERS.n_collider_vertices;
            let len = collider_coordinates.len();

            assert_eq!(n, len as u64);
        });
    }

    #[test]
    fn init_creates_nodes() {
        initialize_storage(None);

        VOICE_NODES_MEMORY.with_borrow(|nodes| {
            assert_eq!(nodes.len(), 360);
            for node in nodes.iter() {
                assert_eq!(node.sample_id, u64::MAX);
            }
        });
    }

    #[test]
    fn init_creates_samples() {
        initialize_storage(None);

        SAMPLES_MEMORY.with_borrow(|samples| {
            assert_eq!(samples.len(), 360);
            for sample in samples.iter() {
                assert_eq!(
                    sample.sample_length_ms as u32,
                    AUDIO_PARAMETERS.max_sample_length_ms
                );
            }
        });
    }

    #[test]
    fn init_creates_zero_cache() {
        initialize_storage(None);

        ZERO_DEGREE_FILE_CACHE.with_borrow(|cache| {
            assert!(cache.len() > 0);
        });
    }
}
