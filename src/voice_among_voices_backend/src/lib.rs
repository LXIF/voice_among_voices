use candid::{CandidType, Principal};
use ic_cdk::{query, update};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap, time::Instant};

#[derive(Debug)]
struct User {
    voice_node: VoiceNode,
    signup_timestamp: Instant,
}

type UserStore = BTreeMap<Principal, User>;

#[derive(Clone, Debug, Deserialize, CandidType)]
struct VoiceNode {
    id: u64,
    x: u64,
    y: u64,
    sample: String, // TODO: update with audio type when we get to it. probably use hound crate. Might make sense to keep audio files separately or only return positions to FE + compiled audio
}

type VoiceNodeStore = Vec<VoiceNode>;

#[derive(Debug)]
struct NFTMap; // TODO: this is one of the last things to implement to make the whole thing NFT-compliant.

#[derive(Debug)]
struct HistoryFrame {
    timestamp: Instant,
    nodes_states: Vec<VoiceNode>,
}

thread_local! {
    static USERS: RefCell<UserStore> = RefCell::new(BTreeMap::new()); //TODO: check how init and pre/post upgrade affect this
    static VOICE_NODES: RefCell<VoiceNodeStore> = RefCell::new(vec![]);
    static HISTORY: RefCell<Vec<HistoryFrame>> = RefCell::new(vec![]);
}

#[update]
fn add_voice_node(node: VoiceNode) {
    VOICE_NODES.with(|nodes| {
        nodes.borrow_mut().push(node);
    });
}

#[query]
fn voice_nodes() -> VoiceNodeStore {
    VOICE_NODES.with(|nodes| nodes.borrow().clone())
}
