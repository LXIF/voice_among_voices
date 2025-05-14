use candid::{CandidType, Decode, Encode};
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

impl Storable for VoiceLog {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap()) // TODO: perhaps more graceful handling
    }
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded; // could become bounded but not sure what the actual size is
}

#[derive(CandidType, Serialize, Deserialize)]
pub enum VoiceAction {
    Drop,
    Censor,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct PositionLog {
    x: f64,
    y: f64,
}
