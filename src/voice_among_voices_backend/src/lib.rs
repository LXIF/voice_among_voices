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

#[derive(Clone, Debug, Deserialize, CandidType)]
struct VoiceNodeLocal {
    id: usize,
    x: f64,
    y: f64,
    sample: String, // TODO: update with audio type when we get to it. probably use hound crate. Might make sense to keep audio files separately or only return positions to FE + compiled audio
}

type VoiceNodeStore = Vec<VoiceNodeLocal>;
type VoiceNodeEgressStore = Vec<VoiceNodeEgress>;

#[derive(Debug)]
struct NFTMap; // TODO: this is one of the last things to implement to make the whole thing NFT-compliant.

#[derive(Debug)]
struct HistoryFrame {
    timestamp: Instant,
    nodes_states: Vec<VoiceNodeLocal>,
}

thread_local! {
    static USERS: RefCell<UserStore> = RefCell::new(BTreeMap::new()); //TODO: check how init and pre/post upgrade affect this
    static VOICE_NODES: RefCell<VoiceNodeStore> = RefCell::new(vec![]);
    static HISTORY: RefCell<Vec<HistoryFrame>> = RefCell::new(vec![]);
}

#[update]
fn add_voice_node(node: VoiceNodeIngress) {
    VOICE_NODES.with(|nodes| {
        let id = nodes.borrow().len().into();
        let new_node = VoiceNodeLocal {
            id,
            x: node.x,
            y: node.y,
            sample: node.sample,
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
