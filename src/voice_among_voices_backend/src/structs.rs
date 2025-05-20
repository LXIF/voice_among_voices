use candid::{define_function, CandidType, Decode, Encode, Principal};
use ic_http_certification::HeaderField;
use ic_stable_structures::{
    memory_manager::VirtualMemory, storable::Bound, DefaultMemoryImpl, StableVec, Storable,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::{borrow::Cow, collections::HashMap, ops::Add};

// LIB ////////////////////

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct VoiceNodeIngress {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub sample: Vec<u8>, // here it's still a blob
}

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct VoiceNodeEgress {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

impl From<VoiceNodeLocal> for VoiceNodeEgress {
    fn from(local: VoiceNodeLocal) -> Self {
        VoiceNodeEgress {
            id: local.id,
            x: local.x,
            y: local.y,
            radius: local.radius,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VoiceNodeLocal {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub sample_id: u64,
    pub radius: f64,
    pub sample_length_samples: u32,
}

impl Storable for VoiceNodeLocal {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap()) // TODO: perhaps more graceful handling
    }
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 79,
        is_fixed_size: true,
    };
}

pub type VoiceNodeLocalStore = Vec<VoiceNodeLocal>;
pub type VoiceNodeLocalMemory = StableVec<VoiceNodeLocal, Memory>;
pub type VoiceNodeEgressStore = Vec<VoiceNodeEgress>;

#[derive(Debug, CandidType, Clone, Deserialize, Serialize)]
pub struct AudioSample {
    pub id: u64,
    pub sample: Vec<u8>,
    pub sample_length_ms: f64,
    pub sample_length_samples: u32,
}

impl Storable for AudioSample {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap()) // TODO: perhaps more graceful handling
    }
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 1024 * 1024, // TODO: calculate from max sample size
        is_fixed_size: false,
    };
}

pub type AudioSampleMemory = StableVec<AudioSample, Memory>;

pub type FileCache = HashMap<u32, Vec<Vec<u8>>>;

#[derive(Debug, Clone, Copy, CandidType, Deserialize)]
pub struct SimulationParameters {
    pub velocity_cutoff: f64,
    pub force_cutoff: f64,
    pub max_distance: f64,
    pub force_strength: f64,
    pub linear_damping: f64,
    pub logical_radius: f64,
    pub n_collider_vertices: u64,
    pub friction: f64,
    pub density: f64,
    pub max_steps: u64,
}

#[derive(Debug, Clone, Copy, CandidType)]
pub struct AudioParameters {
    pub total_length_ms: u32,
    pub max_sample_length_ms: u32,
    pub sample_rate: u32,
    pub chunk_size: usize,
    pub fade_ms: u32,
}

#[derive(CandidType, Debug)]
pub enum AddVoiceNodeError {
    NotWithinCircleError(String),
    NotValidAudioFileError(String),
    Unauthorized,
    EvmError(String),
    SetupError(String),
}

impl From<AuthorizationError> for AddVoiceNodeError {
    fn from(error: AuthorizationError) -> Self {
        match error {
            AuthorizationError::Unauthorized => AddVoiceNodeError::Unauthorized,
            AuthorizationError::EvmError(e) => AddVoiceNodeError::EvmError(e),
            AuthorizationError::SetupError(e) => AddVoiceNodeError::SetupError(e),
        }
    }
}

#[derive(CandidType, Debug)]
pub enum AuthorizationError {
    Unauthorized,
    EvmError(String),
    SetupError(String),
}

#[derive(CandidType, Debug)]
pub enum CensorshipError {
    Unauthorized,
    EvmError(String),
    SetupError(String),
    VoiceNotFound,
    InternalCanisterError(String),
}

impl From<AuthorizationError> for CensorshipError {
    fn from(error: AuthorizationError) -> Self {
        match error {
            AuthorizationError::Unauthorized => CensorshipError::Unauthorized,
            AuthorizationError::EvmError(e) => CensorshipError::EvmError(e),
            AuthorizationError::SetupError(e) => CensorshipError::SetupError(e),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Default)]
pub struct HttpStreamingResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub upgrade: Option<bool>,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StreamingCallbackToken {
    pub angle: u32,
    pub chunk_index: u32,
    pub chunks: u32,
    pub auth_token: Option<ByteBuf>,
}

impl StreamingCallbackToken {
    pub fn next(self) -> Option<StreamingCallbackToken> {
        if self.chunk_index + 1 >= self.chunks {
            None
        } else {
            Some(StreamingCallbackToken {
                angle: self.angle,
                chunk_index: self.chunk_index + 1,
                chunks: self.chunks,
                auth_token: self.auth_token,
            })
        }
    }
}

define_function!(pub CallbackFunc : (StreamingCallbackToken) -> (StreamingCallbackHttpResponse) query);

#[derive(CandidType, Deserialize, Clone)]
pub enum StreamingStrategy {
    Callback {
        token: StreamingCallbackToken,
        callback: CallbackFunc,
    },
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StreamingCallbackHttpResponse {
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub token: Option<StreamingCallbackToken>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Default, Eq, PartialEq)]
pub struct VoiceAmongVoicesInit {
    pub siwe_canister_principal: Option<Principal>,
    pub token_address: Option<String>,
    pub dev_mode: Option<bool>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CandidType, Serialize, Deserialize,
)]
pub struct StorableConfig {
    pub dev_mode: bool,
    pub admin_id: u64,
}

impl StorableConfig {
    pub fn default() -> Self {
        StorableConfig {
            dev_mode: false,
            admin_id: 0,
        }
    }
}

impl Storable for StorableConfig {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap()) // TODO: perhaps more graceful handling
    }
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Unbounded;
}
