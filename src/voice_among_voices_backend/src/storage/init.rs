use crate::{
    audio::precompute_fade_curves, create_circular_collider_coordinates, storage::FADES,
    store_siwe_principal, store_token_address, structs::StorableAudioSample,
    test_functions::generate_test_sample_vec, zero_cache_update, VoiceAmongVoicesInit,
    VoiceNodeLocal, AUDIO_PARAMETERS, COLLIDER_COORDINATES, SAMPLES_MEMORY, SIMULATION_PARAMETERS,
    VOICE_NODES_MEMORY,
};
use alloy::primitives::Address;
use std::str::FromStr;

use super::{store_admin_token_id, store_dev_mode, store_token_buy_link};

// abstracting this because during dev things change and i don't want to restart dfx all the time
pub fn collider_init() {
    COLLIDER_COORDINATES.with_borrow_mut(|collider_coordinates| {
        let fresh_vertices = create_circular_collider_coordinates(
            SIMULATION_PARAMETERS.n_collider_vertices,
            SIMULATION_PARAMETERS.logical_radius,
        );

        collider_coordinates.extend(fresh_vertices);
    });
}

pub fn fades_init() {
    FADES.with_borrow_mut(|fade| {
        let fade_samples =
            (AUDIO_PARAMETERS.fade_ms * AUDIO_PARAMETERS.sample_rate / 1000) as usize;
        let fades_cache = precompute_fade_curves(fade_samples);

        fade.fade_in = fades_cache.fade_in;
        fade.fade_out = fades_cache.fade_out;
    });
}

pub fn nodes_init() {
    VOICE_NODES_MEMORY.with_borrow_mut(|nodes| {
        for i in 0..360 {
            nodes
                .push(&VoiceNodeLocal {
                    id: i,
                    x: 0.,
                    y: 0.,
                    sample_id: u64::MAX,
                    radius: f64::MAX,
                    sample_length_samples: u32::MAX,
                })
                .expect("Failed to initialize voice nodes");
        }
    });
}

pub fn samples_init() {
    SAMPLES_MEMORY.with_borrow_mut(|samples| {
        let sample_length_ms = AUDIO_PARAMETERS.max_sample_length_ms;
        let sample_rate = AUDIO_PARAMETERS.sample_rate;
        let start_sample = generate_test_sample_vec(sample_length_ms, sample_rate, 1.);
        for i in 0..360 {
            samples
                .push(&StorableAudioSample {
                    id: i,
                    sample: start_sample.clone(),
                    sample_length_ms: sample_length_ms as f64,
                    sample_length_samples: sample_length_ms * sample_rate / 1000,
                })
                .expect("Failed to initialize samples");
        }
    });
}

pub fn zero_cache_init() {
    zero_cache_update();
}

pub fn initialize_storage(maybe_arg: Option<VoiceAmongVoicesInit>) {
    nodes_init();
    samples_init();

    upgrade_storage(maybe_arg);
}

pub fn upgrade_storage(maybe_arg: Option<VoiceAmongVoicesInit>) {
    collider_init();
    fades_init();
    zero_cache_init();

    if let Some(args) = maybe_arg {
        if let Some(siwe_principal) = args.siwe_canister_principal {
            let _ = store_siwe_principal(siwe_principal);
        }
        if let Some(token_address) = args.token_address {
            let parsed_address =
                Address::from_str(&token_address).expect("Could not parse token address");
            let _ = store_token_address(parsed_address);
        }
        if let Some(token_buy_link) = args.token_buy_link {
            let _ = store_token_buy_link(token_buy_link);
        }
        if let Some(dev_mode) = args.dev_mode {
            let _ = store_dev_mode(dev_mode);
        }
        if let Some(admin_token_id) = args.admin_token_id {
            let _ = store_admin_token_id(admin_token_id);
        }
    }
}
