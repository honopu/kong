/// generated by didc on WUMBO canister: wkv3f-iiaaa-aaaap-ag73a-cai
///
/// Transaction1 used for get_transaction()
use candid::{CandidType, Nat};
use icrc_ledger_types::icrc1::account::Account;
use serde::Deserialize;

pub type Balance = Nat;

pub type TxIndex = Nat;
pub type Timestamp = u64;

#[derive(CandidType, Deserialize)]
pub struct Burn {
    pub from: Account,
    pub memo: Option<serde_bytes::ByteBuf>,
    pub created_at_time: Option<u64>,
    pub amount: Balance,
}

#[derive(CandidType, Deserialize)]
pub struct Transaction1 {
    pub burn: Option<Burn>,
    pub kind: String,
    pub mint: Option<Mint1>,
    pub timestamp: Timestamp,
    pub index: TxIndex,
    pub transfer: Option<Transfer>,
}

#[derive(CandidType, Deserialize)]
pub struct Mint1 {
    pub to: Account,
    pub memo: Option<serde_bytes::ByteBuf>,
    pub created_at_time: Option<u64>,
    pub amount: Balance,
}
#[derive(CandidType, Deserialize)]
pub struct Transfer {
    pub to: Account,
    pub fee: Option<Balance>,
    pub from: Account,
    pub memo: Option<serde_bytes::ByteBuf>,
    pub created_at_time: Option<u64>,
    pub amount: Balance,
}
