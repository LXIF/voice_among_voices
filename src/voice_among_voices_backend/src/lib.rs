mod physics;

use candid::{CandidType, Principal};
use ic_cdk::{query, update};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap, time::Instant};

#[derive(Debug)]
struct User {
    voice_node: VoiceNodeLocal,
    signup_timestamp: Instant,
}

type UserStore = BTreeMap<Principal, User>;

#[derive(Clone, Debug, Deserialize, CandidType)]
struct VoiceNodeIngress {
    x: f64,
    y: f64,
    sample: String, // TODO: update with audio type when we get to it. probably use hound crate. Might make sense to keep audio files separately or only return positions to FE + compiled audio
}

#[derive(Clone, Debug, Deserialize, CandidType)]
struct VoiceNodeEgress {
    id: usize,
    x: f64,
    y: f64,
}

impl From<VoiceNodeLocal> for VoiceNodeEgress {
    fn from(local: VoiceNodeLocal) -> Self {
        VoiceNodeEgress {
            id: local.id,
            x: local.x,
            y: local.y,
        }
    }
}

#[derive(Clone, Debug, CandidType)]
struct VoiceNodeLocal {
    id: usize,
    x: f64,
    y: f64,
    sample_id: usize,
}

type VoiceNodeLocalStore = Vec<VoiceNodeLocal>;
type VoiceNodeEgressStore = Vec<VoiceNodeEgress>;

#[derive(Debug, CandidType)]
struct AudioSample {
    id: usize,
    sample: String, // TODO: replace this with audio type
}

type AudioSampleStore = Vec<AudioSample>;

#[derive(Debug)]
struct NFTMap; // TODO: this is one of the last things to implement to make the whole thing NFT-compliant.

#[derive(Debug)]
struct HistoryFrame {
    timestamp: Instant,
    nodes_states: Vec<VoiceNodeLocal>,
}

#[derive(Debug, Clone, Copy, CandidType)]
struct SimulationParameters {
    velocity_cutoff: f64,
    force_cutoff: f64,
    max_distance: f64,
    force_strength: f64,
    linear_damping: f64,
    logical_width: f64,
    logical_height: f64,
}

thread_local! { // TODO: replace with stable structures and make auto-scaling
    static USERS: RefCell<UserStore> = RefCell::new(BTreeMap::new()); //TODO: check how init and pre/post upgrade affect this
    static VOICE_NODES: RefCell<VoiceNodeLocalStore> = RefCell::new(vec![]);
    static SAMPLES: RefCell<AudioSampleStore> = RefCell::new(vec![]);
    static HISTORY: RefCell<Vec<HistoryFrame>> = RefCell::new(vec![]);
    static SIMULATION_PARAMETERS: SimulationParameters = SimulationParameters {
        velocity_cutoff: 0.2,
        force_cutoff: 100.,
        max_distance: 20.,
        force_strength: 3000.,
        linear_damping: 10.,
        logical_height: 100.,
        logical_width: 100.
    }
}

#[update]
fn add_voice_node(node: VoiceNodeIngress) {
    let mut sample_id = 0;
    SAMPLES.with(|samples| {
        let new_sample = AudioSample {
            id: samples.borrow().len(),
            sample: node.sample,
        };
        sample_id = new_sample.id.clone();
        samples.borrow_mut().push(new_sample);
    });

    VOICE_NODES.with(|nodes| {
        let id = nodes.borrow().len().into();
        let new_node = VoiceNodeLocal {
            id,
            x: node.x,
            y: node.y,
            sample_id,
        };

        nodes.borrow_mut().push(new_node);
    });
}

#[query]
fn voice_nodes() -> VoiceNodeEgressStore {
    VOICE_NODES.with(|nodes| {
        nodes
            .borrow()
            .clone()
            .into_iter()
            .map(|node| node.into())
            .collect()
    })
}

#[query]
fn get_simulation_parameters() -> SimulationParameters {
    SIMULATION_PARAMETERS.with(|params| params.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_nodes_get_added_correctly() {
        let voice_node = VoiceNodeIngress {
            x: 10.,
            y: 10.,
            sample: "wargle".to_string(),
        };

        let another_voice_node = VoiceNodeIngress {
            x: 10.,
            y: 10.,
            sample: "wargle".to_string(),
        };

        add_voice_node(voice_node);
        add_voice_node(another_voice_node);

        SAMPLES.with(|samples| {
            let id = samples.borrow()[0].id;
            let another_id = samples.borrow()[1].id;

            assert_eq!(id, 0);
            assert_eq!(another_id, 1);
        });
    }
}
