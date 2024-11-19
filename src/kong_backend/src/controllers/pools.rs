use candid::Nat;
use ic_cdk::{query, update};
use std::collections::BTreeMap;

use crate::ic::guards::caller_is_kingkong;
use crate::stable_memory::POOL_MAP;
use crate::stable_pool::pool_map;
use crate::stable_pool::stable_pool::{StablePool, StablePoolId};

const MAX_POOLS: usize = 1_000;

/// serializes POOL_MAP for backup
#[query(hidden = true, guard = "caller_is_kingkong")]
fn backup_pools(pool_id: Option<u32>, num_pools: Option<u16>) -> Result<String, String> {
    POOL_MAP.with(|m| {
        let map = m.borrow();
        let pools: BTreeMap<_, _> = match pool_id {
            Some(pool_id) => {
                let start_id = StablePoolId(pool_id);
                let num_pools = num_pools.map_or(1, |n| n as usize);
                map.range(start_id..).take(num_pools).collect()
            }
            None => {
                let num_pools = num_pools.map_or(MAX_POOLS, |n| n as usize);
                map.iter().take(num_pools).collect()
            }
        };
        serde_json::to_string(&pools).map_err(|e| format!("Failed to serialize pools: {}", e))
    })
}

/// deserialize POOL_MAP and update stable memory
#[update(hidden = true, guard = "caller_is_kingkong")]
fn update_pools(tokens: String) -> Result<String, String> {
    let pools: BTreeMap<StablePoolId, StablePool> = match serde_json::from_str(&tokens) {
        Ok(pools) => pools,
        Err(e) => return Err(format!("Invalid pools: {}", e)),
    };

    POOL_MAP.with(|user_map| {
        let mut map = user_map.borrow_mut();
        for (k, v) in pools {
            map.insert(k, v);
        }
    });

    Ok("Pools updated".to_string())
}

#[update(hidden = true, guard = "caller_is_kingkong")]
fn update_pool(symbol: String, balance_0: Nat, balance_1: Nat) -> Result<String, String> {
    let mut pool = pool_map::get_by_token(&symbol)?;
    pool.balance_0 = balance_0;
    pool.balance_1 = balance_1;
    _ = pool_map::update(&pool);

    Ok(format!("Pool {} updated", symbol))
}

#[update(hidden = true, guard = "caller_is_kingkong")]
fn remove_pool(symbol: String) -> Result<String, String> {
    let pool = pool_map::get_by_token(&symbol)?;
    pool_map::remove(&pool)?;

    Ok(format!("Pool {} removed", symbol))
}

/*
#[update(hidden = true, guard = "caller_is_kingkong")]
fn upgrade_pools() -> Result<String, String> {
    POOL_ALT_MAP.with(|m| {
        let pool_alt_map = m.borrow();
        POOL_MAP.with(|m| {
            let mut pool_map = m.borrow_mut();
            pool_map.clear_new();
            for (k, v) in pool_alt_map.iter() {
                let pool_id = StablePoolIdAlt::to_stable_pool_id(&k);
                let pool = StablePoolAlt::to_stable_pool(&v);
                pool_map.insert(pool_id, pool);
            }
        });
    });

    Ok("Pools upgraded".to_string())
}

#[update(hidden = true, guard = "caller_is_kingkong")]
fn upgrade_alt_pools() -> Result<String, String> {
    POOL_MAP.with(|m| {
        let pool_map = m.borrow();
        POOL_ALT_MAP.with(|m| {
            let mut pool_alt_map = m.borrow_mut();
            pool_alt_map.clear_new();
            for (k, v) in pool_map.iter() {
                let pool_id = StablePoolIdAlt::from_stable_pool_id(&k);
                let pool = StablePoolAlt::from_stable_pool(&v);
                pool_alt_map.insert(pool_id, pool);
            }
        });
    });

    Ok("Alt pools upgraded".to_string())
}
*/
