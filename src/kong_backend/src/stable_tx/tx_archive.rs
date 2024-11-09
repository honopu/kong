use crate::ic::get_time::get_time;
use crate::ic::guards::not_in_maintenance_mode;
use crate::stable_memory::{TX_24H_MAP, TX_ARCHIVE_MAP, TX_MAP};
use crate::stable_tx::stable_tx::{StableTx, StableTxId};

pub fn archive_tx_map() {
    if not_in_maintenance_mode().is_err() {
        return;
    }

    // archive txs
    TX_MAP.with(|tx_map| {
        TX_ARCHIVE_MAP.with(|tx_archive_map| {
            let tx = tx_map.borrow();
            let mut tx_archive = tx_archive_map.borrow_mut();
            let start_tx_id = tx_archive.last_key_value().map_or(0_u64, |(k, _)| k.0);
            let end_tx_id = tx.last_key_value().map_or(0_u64, |(k, _)| k.0);
            for tx_id in start_tx_id..=end_tx_id {
                if let Some(tx) = tx.get(&StableTxId(tx_id)) {
                    tx_archive.insert(StableTxId(tx_id), tx);
                }
            }
        });
    });
}

pub fn archive_tx_24h_map() {
    if not_in_maintenance_mode().is_err() {
        return;
    }

    let ts_start = get_time() - 86_400_000_000_000; // 24 hours
    TX_MAP.with(|tx_map| {
        let map = tx_map.borrow();
        TX_24H_MAP.with(|tx_24h_map| {
            let mut map_24h = tx_24h_map.borrow_mut();
            map_24h.clear_new();

            for (tx_id, tx) in map.iter() {
                if let StableTx::Swap(swap_tx) = tx.clone() {
                    if swap_tx.ts >= ts_start {
                        map_24h.insert(tx_id, tx);
                    }
                }
            }
        });
    });
}
