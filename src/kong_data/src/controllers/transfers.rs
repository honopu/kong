use ic_cdk::{query, update};
use std::collections::BTreeMap;

use crate::ic::get_time::get_time;
use crate::ic::guards::{caller_is_kingkong, caller_is_kong_backend};
use crate::stable_db_update::db_update_map;
use crate::stable_db_update::stable_db_update::{StableDBUpdate, StableMemory};
use crate::stable_memory::TRANSFER_MAP;
use crate::stable_transfer::stable_transfer::{StableTransfer, StableTransferId};

const MAX_TRANSFERS: usize = 1_000;

#[query(hidden = true, guard = "caller_is_kingkong")]
fn max_transfer_idx() -> u64 {
    TRANSFER_MAP.with(|m| m.borrow().last_key_value().map_or(0, |(k, _)| k.0))
}

#[query(hidden = true, guard = "caller_is_kingkong")]
fn backup_transfers(transfer_id: Option<u64>, num_requests: Option<u16>) -> Result<String, String> {
    TRANSFER_MAP.with(|m| {
        let map = m.borrow();
        let transfers: BTreeMap<_, _> = match transfer_id {
            Some(transfer_id) => {
                let num_requests = num_requests.map_or(1, |n| n as usize);
                let start_key = StableTransferId(transfer_id);
                map.range(start_key..).take(num_requests).collect()
            }
            None => {
                let num_requests = num_requests.map_or(MAX_TRANSFERS, |n| n as usize);
                map.iter().take(num_requests).collect()
            }
        };

        serde_json::to_string(&transfers).map_err(|e| format!("Failed to serialize transfers: {}", e))
    })
}

/// deserialize TRANSFER_MAP and update stable memory
#[update(hidden = true, guard = "caller_is_kingkong")]
fn update_transfers(stable_transfers_json: String) -> Result<String, String> {
    let transfers: BTreeMap<StableTransferId, StableTransfer> = match serde_json::from_str(&stable_transfers_json) {
        Ok(transfers) => transfers,
        Err(e) => return Err(format!("Invalid transfers: {}", e)),
    };

    TRANSFER_MAP.with(|transfer_map| {
        let mut map = transfer_map.borrow_mut();
        for (k, v) in transfers {
            map.insert(k, v);
        }
    });

    Ok("Transfers updated".to_string())
}

#[update(hidden = true, guard = "caller_is_kong_backend")]
fn update_transfer(stable_transfer_json: String) -> Result<String, String> {
    let transfer: StableTransfer = match serde_json::from_str(&stable_transfer_json) {
        Ok(transfer) => transfer,
        Err(e) => return Err(format!("Invalid transfer: {}", e)),
    };

    TRANSFER_MAP.with(|transfer_map| {
        let mut map = transfer_map.borrow_mut();
        map.insert(StableTransferId(transfer.transfer_id), transfer.clone());
    });

    // add to UpdateMap for archiving to database
    let ts = get_time();
    let update = StableDBUpdate {
        db_update_id: 0,
        stable_memory: StableMemory::TransferMap(transfer),
        ts,
    };
    db_update_map::insert(&update);

    Ok("Transfer updated".to_string())
}
