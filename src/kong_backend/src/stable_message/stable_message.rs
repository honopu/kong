use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;
use std::borrow::Cow;

const MESSAGE_ID_SIZE: u32 = std::mem::size_of::<u64>() as u32;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize)]
pub struct StableMessageId(pub u64);

impl Storable for StableMessageId {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        self.0.to_bytes() // u64 is already Storable
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(u64::from_bytes(bytes))
    }

    // u64 is fixed size
    const BOUND: Bound = Bound::Bounded {
        max_size: MESSAGE_ID_SIZE,
        is_fixed_size: true,
    };
}

#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub struct StableMessage {
    pub message_id: u64, // unique id (same as StableMessageId) for MESSAGE_MAP
    pub to_user_id: u32, // user id of receiver
    pub title: String,   // title
    pub message: String, // message
    pub ts: u64,         // timestamp
}

impl StableMessage {
    pub fn new(to_user_id: u32, title: &str, message: &str, ts: u64) -> Self {
        Self {
            message_id: 0,
            to_user_id,
            title: title.to_string(),
            message: message.to_string(),
            ts,
        }
    }
}

impl Storable for StableMessage {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    // unbounded size
    const BOUND: Bound = Bound::Unbounded;
}
