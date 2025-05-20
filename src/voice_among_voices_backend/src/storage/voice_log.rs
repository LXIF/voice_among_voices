use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use super::AddressEgress;

#[derive(CandidType, Serialize, Deserialize)]
pub struct VoiceLog {
    timestamp: u64, // nanos since epoch
    id: u64,
    action: VoiceAction,
    initiator: AddressEgress,
    position: Option<PositionLog>,
}

impl VoiceLog {
    pub fn new(
        timestamp: u64,
        id: u64,
        action: VoiceAction,
        initiator: AddressEgress,
        position: Option<PositionLog>,
    ) -> Self {
        Self {
            timestamp,
            id,
            action,
            initiator,
            position,
        }
    }
}

impl Storable for VoiceLog {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 256,
        is_fixed_size: false,
    }; // TODO: max_size may be false
}

#[derive(CandidType, Serialize, Deserialize)]
pub enum VoiceAction {
    Drop,
    Censor,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct PositionLog {
    pub x: f64,
    pub y: f64,
}
