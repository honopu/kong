use candid::{Nat, Principal};
use ic_cdk::{query, update};
use icrc_ledger_types::icrc1::account::Account;
use std::collections::BTreeMap;

use crate::helpers::nat_helpers::{nat_add, nat_subtract, nat_zero};
use crate::ic::guards::caller_is_kingkong;
use crate::remove_liquidity::remove_liquidity::remove_liquidity_from_pool;
use crate::remove_liquidity::remove_liquidity_args::RemoveLiquidityArgs;
use crate::stable_lp_token::lp_token_map;
use crate::stable_memory::{LP_TOKEN_MAP, POOL_MAP};
use crate::stable_pool::stable_pool::{StablePool, StablePoolId};
use crate::stable_pool::{pool_map, pool_stats};
use crate::stable_token::token::Token;
use crate::stable_user::user_map;

const MAX_POOLS: usize = 1_000;

#[query(hidden = true, guard = "caller_is_kingkong")]
fn max_pool_idx() -> u32 {
    POOL_MAP.with(|m| m.borrow().last_key_value().map_or(0, |(k, _)| k.0))
}

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

    for (_, v) in pools {
        pool_map::update(&v);
    }

    Ok("Pools updated".to_string())
}

// remove all LP positions from pool, returning all tokens to users
#[update(hidden = true, guard = "caller_is_kingkong")]
async fn remove_lps_from_pool(symbol: String) -> Result<String, String> {
    let pool = pool_map::get_by_token(&symbol)?;
    let lp_token_id = pool.lp_token_id;

    // list of all LP positions to remove
    // (user_id, principal_id, lp token amount)
    let lp_users = LP_TOKEN_MAP.with(|m| {
        m.borrow()
            .iter()
            .filter_map(|(_, v)| {
                if v.token_id == lp_token_id {
                    let user = user_map::get_by_user_id(v.user_id)?;
                    Some((user.user_id, user.principal_id, v.amount))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    });

    // remove_liquidity for each user
    let token_0 = pool.token_0().address_with_chain();
    let token_1 = pool.token_1().address_with_chain();
    let mut results = Vec::new();
    for (user_id, principal_id, remove_lp_token_amount) in lp_users {
        // skip if user has no LP position
        if remove_lp_token_amount == nat_zero() {
            continue;
        }
        let args = RemoveLiquidityArgs {
            token_0: token_0.clone(),
            token_1: token_1.clone(),
            remove_lp_token_amount,
        };
        match Principal::from_text(principal_id) {
            Ok(principal) => {
                let to_principal_id = Account::from(principal);
                match remove_liquidity_from_pool(args, user_id, &to_principal_id).await {
                    Ok(_) => {
                        results.push(format!("Removed user_id {} LP position", user_id));
                    }
                    Err(e) => {
                        results.push(format!("Failed to remove user_id {} LP position: {}", user_id, e));
                    }
                }
            }
            Err(e) => {
                results.push(format!("Failed to parse user_id {} principal id: {}", user_id, e));
            }
        }
    }

    // check remaining LP token total supply
    let lp_total_supply = lp_token_map::get_total_supply(lp_token_id);
    results.push(format!("Remaining LP token total supply {}", lp_total_supply));

    serde_json::to_string(&results).map_err(|e| format!("Failed to serialize remove_liquidity: {}", e))
}

/// remove pool, token, LP token and all LP positions
#[update(hidden = true, guard = "caller_is_kingkong")]
fn remove_pool(symbol: String) -> Result<String, String> {
    let pool = pool_map::get_by_token(&symbol)?;
    let lp_token_id = pool.lp_token_id;
    let lp_total_supply = lp_token_map::get_total_supply(lp_token_id);
    if lp_total_supply > nat_zero() {
        return Err(format!("LP token total supply is still {}", lp_total_supply));
    }

    pool_map::remove(pool.pool_id)?;

    Ok(format!("Pool {} removed", symbol))
}

/// suspend pool, set is_removed to true
#[update(hidden = true, guard = "caller_is_kingkong")]
fn suspend_pool(symbol: String) -> Result<String, String> {
    let pool = pool_map::get_by_token(&symbol)?;
    pool_map::remove(pool.pool_id)?;

    Ok(format!("Pool {} suspended", symbol))
}

#[update(hidden = true, guard = "caller_is_kingkong")]
fn unsuspend_pool(symbol: String) -> Result<String, String> {
    let pool = pool_map::get_by_token(&symbol)?;
    pool_map::unremove(pool.pool_id)?;

    Ok(format!("Pool {} unsuspended", symbol))
}

/// used to force a pool stats update
#[update(hidden = true, guard = "caller_is_kingkong")]
fn update_pool_stats() -> Result<String, String> {
    pool_stats::update_pool_stats()?;

    Ok("Pool stats updated".to_string())
}

/// used to force a pool tvl update
#[update(hidden = true, guard = "caller_is_kingkong")]
fn update_pool_tvl(symbol: String) -> Result<String, String> {
    let mut pool = pool_map::get_by_token(&symbol)?;
    pool.set_tvl();
    pool_map::update(&pool);

    serde_json::to_string(&pool).map_err(|e| format!("Failed to serialize: {}", e))
}

#[update(hidden = true, guard = "caller_is_kingkong")]
fn g20_payment() -> Result<String, String> {
    let mut icp_ckusdt = pool_map::get_by_token("ICP_ckUSDT")?;
    let amount_0 = Nat::from(311_341_379_409_u128);
    let amount_1 = Nat::from(22_215_014_188_u128);
    icp_ckusdt.balance_0 = nat_add(&icp_ckusdt.balance_0, &amount_0);
    icp_ckusdt.balance_1 = nat_add(&icp_ckusdt.balance_1, &amount_1);
    pool_map::update(&icp_ckusdt);
    println!("ICP_ckUSDT updated");

    let mut ckbtc_ckusdt = pool_map::get_by_token("ckBTC_ckUSDT")?;
    let amount_0 = Nat::from(260_447_u128);
    let amount_1 = Nat::from(250_405_302_u128);
    ckbtc_ckusdt.balance_0 = nat_subtract(&ckbtc_ckusdt.balance_0, &amount_0).unwrap();
    ckbtc_ckusdt.balance_1 = nat_subtract(&ckbtc_ckusdt.balance_1, &amount_1).unwrap();
    pool_map::update(&ckbtc_ckusdt);
    println!("ckBTC_ckUSDT updated");

    let mut ckusdc_ckusdt = pool_map::get_by_token("ckUSDC_ckUSDT")?;
    let amount_0 = Nat::from(20_625_749_630_u128);
    let amount_1 = Nat::from(20_634_769_003_u128);
    ckusdc_ckusdt.balance_0 = nat_subtract(&ckusdc_ckusdt.balance_0, &amount_0).unwrap();
    ckusdc_ckusdt.balance_1 = nat_subtract(&ckusdc_ckusdt.balance_1, &amount_1).unwrap();
    pool_map::update(&ckusdc_ckusdt);
    println!("ckUSDC_ckUSDT updated");

    let mut cketh_ckusdt = pool_map::get_by_token("ckETH_ckUSDT")?;
    let amount_0 = Nat::from(16_330_000_000_000_000_000_u128);
    let fee_0 = Nat::from(108_056_736_117_801_000_u128);
    let amount_1 = Nat::from(44_078_851_681_u128);
    cketh_ckusdt.balance_0 = nat_subtract(&cketh_ckusdt.balance_0, &amount_0).unwrap();
    cketh_ckusdt.lp_fee_0 = nat_subtract(&cketh_ckusdt.lp_fee_0, &fee_0).unwrap();
    cketh_ckusdt.balance_1 = nat_subtract(&cketh_ckusdt.balance_1, &amount_1).unwrap();
    pool_map::update(&cketh_ckusdt);
    println!("ckETH_ckUSDT updated");

    Ok("Pools updated with g20 payments".to_string())
}
