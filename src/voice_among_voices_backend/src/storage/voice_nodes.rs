use std::time::Duration;

use alloy::primitives::Address;
use ic_cdk::api::time;

use crate::physics::simulate_until_stopped;
use crate::storage::{
    AUDIO_PARAMETERS, COLLIDER_COORDINATES, SAMPLES_MEMORY, SIMULATION_PARAMETERS,
    VOICE_NODES_MEMORY,
};
use crate::{
    get_sample_length, node_within_circle, AddVoiceNodeError, AudioSample, VoiceNodeEgressStore,
    VoiceNodeIngress, VoiceNodeLocal,
};

use super::voice_log::{PositionLog, VoiceAction, VoiceLog};
use super::{store_voice_log, zero_cache_update};

pub fn update_stored_voice_node(
    node: VoiceNodeIngress,
    address: Address, // needed for logging
) -> Result<VoiceNodeEgressStore, AddVoiceNodeError> {
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

    SAMPLES_MEMORY.with_borrow_mut(|samples_map| {
        let new_sample = AudioSample {
            id: node.id as u64,
            sample: node.sample,
            sample_length_samples,
            sample_length_ms,
        };
        samples_map.set(node.id as u64, &new_sample);
    });

    let mut returnable_nodes: VoiceNodeEgressStore = vec![];

    COLLIDER_COORDINATES.with_borrow(|collider_coordinates| {
        VOICE_NODES_MEMORY.with_borrow_mut(|nodes| {
            let id = node.id;
            let new_node = VoiceNodeLocal {
                id,
                x: node.x,
                y: node.y,
                sample_id: node.id as u64,
                radius: node_radius,
                sample_length_samples,
            };

            nodes.set(id as u64, &new_node);

            simulate_until_stopped(nodes, &SIMULATION_PARAMETERS, &collider_coordinates);

            returnable_nodes = nodes
                .iter()
                .filter_map(|node| {
                    if node.sample_id == u64::MAX {
                        return None;
                    }
                    Some(node.into())
                })
                .collect();
        });
    });

    let _ = store_voice_log(VoiceLog::new(
        //TODO: maybe handle better
        time(),
        node.id as u64,
        VoiceAction::Drop,
        address.into(),
        Some(PositionLog {
            x: node.x,
            y: node.y,
        }),
    ));

    // zero_cache_update();
    ic_cdk_timers::set_timer(Duration::from_nanos(1), zero_cache_update);

    Ok(returnable_nodes)
}

pub fn get_stored_voice_nodes() -> VoiceNodeEgressStore {
    VOICE_NODES_MEMORY.with_borrow(|nodes| {
        nodes
            .iter()
            .filter_map(|node| {
                if node.sample_id == u64::MAX {
                    return None;
                }
                Some(node.into())
            })
            .collect()
    })
}
