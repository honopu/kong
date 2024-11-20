use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};

use super::add_liquidity_tx::AddLiquidityTx;
use super::add_pool_tx::AddPoolTx;
use super::remove_liquidity_tx::RemoveLiquidityTx;
use super::send_tx::SendTx;
use super::stable_tx_old::{StableTxIdOld, StableTxOld};
use super::swap_tx::SwapTx;

#[derive(CandidType, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StableTxId(pub u64);

impl StableTxId {
    pub fn from_old(stable_tx_id: &StableTxIdOld) -> Self {
        let tx_id_old = serde_json::to_value(stable_tx_id).unwrap();
        serde_json::from_value(tx_id_old).unwrap()
    }
}

impl Storable for StableTxId {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        serde_cbor::to_vec(self).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub enum StableTx {
    AddPool(AddPoolTx),
    AddLiquidity(AddLiquidityTx),
    RemoveLiquidity(RemoveLiquidityTx),
    Swap(SwapTx),
    Send(SendTx),
}

impl StableTx {
    pub fn from_old(stable_tx: &StableTxOld) -> Self {
        let tx_old = serde_json::to_value(stable_tx).unwrap();
        serde_json::from_value(tx_old).unwrap()
    }
}

impl Storable for StableTx {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        serde_cbor::to_vec(self).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}
