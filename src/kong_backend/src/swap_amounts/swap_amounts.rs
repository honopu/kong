use candid::Nat;
use ic_cdk::query;

use super::swap_amounts_reply::SwapAmountsReply;
use super::swap_amounts_reply_impl::to_swap_amounts_tx_reply;

use crate::helpers::math_helpers::price_rounded;
use crate::helpers::nat_helpers::nat_zero;
use crate::ic::guards::not_in_maintenance_mode;
use crate::stable_pool::pool_map;
use crate::stable_pool::stable_pool::StablePool;
use crate::stable_token::token::Token;
use crate::stable_token::token_map;
use crate::stable_user::user_map;
use crate::swap::swap_calc::SwapCalc;
use crate::swap::swap_calc_impl::{get_slippage, swap_amount_0, swap_amount_1};

#[query(guard = "not_in_maintenance_mode")]
pub fn swap_amounts(pay_token: String, pay_amount: Nat, receive_token: String) -> Result<SwapAmountsReply, String> {
    // Pay token
    let pay_token = token_map::get_by_token(&pay_token)?;
    let pay_token_id = pay_token.token_id();
    let pay_chain = pay_token.chain();
    let pay_symbol = pay_token.symbol();
    let pay_address = pay_token.address();
    // Receive token
    let receive_token = token_map::get_by_token(&receive_token)?;
    let receive_token_id = receive_token.token_id();
    let receive_chain = receive_token.chain();
    let receive_symbol = receive_token.symbol();
    let receive_address = receive_token.address();

    let user_fee_level = user_map::get_by_caller().ok().flatten().unwrap_or_default().fee_level;
    let mut txs = Vec::new();

    // check if direct pool exists
    if let Some(pool) = pool_map::get_by_token_ids(pay_token_id, receive_token_id) {
        let swap = swap_amount_0(&pool, &pay_amount, Some(user_fee_level), None, None)?;
        txs.push(to_swap_amounts_tx_reply(&swap).ok_or("Invalid swap tokens")?);
        let receive_amount = swap.receive_amount_with_fees_and_gas();
        let price = swap.get_price().ok_or("Invalid price")?;
        let price_f64 = price_rounded(&price).ok_or("Invalid price")?;
        let mid_price = swap.get_mid_price().ok_or("Invalid mid price")?;
        let mid_price_f64 = price_rounded(&mid_price).ok_or("Invalid mid price")?;
        let slippage_f64 = get_slippage(&price, &mid_price).ok_or("Invalid slippage")?;
        return Ok(SwapAmountsReply {
            pay_chain,
            pay_symbol,
            pay_amount,
            pay_address,
            receive_chain,
            receive_symbol,
            receive_address,
            receive_amount,
            price: price_f64,
            txs,
            mid_price: mid_price_f64,
            slippage: slippage_f64,
        });
    };

    if let Some(pool) = pool_map::get_by_token_ids(receive_token_id, pay_token_id) {
        let swap = swap_amount_1(&pool, &pay_amount, Some(user_fee_level), None, None)?;
        txs.push(to_swap_amounts_tx_reply(&swap).ok_or("Invalid swap tokens")?);
        let receive_amount = swap.receive_amount_with_fees_and_gas();
        let price = swap.get_price().ok_or("Invalid price")?;
        let price_f64 = price_rounded(&price).ok_or("Invalid price")?;
        let mid_price = swap.get_mid_price().ok_or("Invalid mid price")?;
        let mid_price_f64 = price_rounded(&mid_price).ok_or("Invalid mid price")?;
        let slippage_f64 = get_slippage(&price, &mid_price).ok_or("Invalid slippage")?;
        return Ok(SwapAmountsReply {
            pay_chain,
            pay_symbol,
            pay_amount,
            pay_address,
            receive_chain,
            receive_symbol,
            receive_address,
            receive_amount,
            price: price_f64,
            txs,
            mid_price: mid_price_f64,
            slippage: slippage_f64,
        });
    };

    // test for 2-step swap via ckUSDT or ICP
    let ckusdt_token_id = token_map::get_ckusdt()?.token_id();
    let icp_token_id = token_map::get_icp()?.token_id();
    let pool1_ckusdt = pool_map::get_by_token_ids(pay_token_id, ckusdt_token_id);
    let pool2_ckusdt = pool_map::get_by_token_ids(receive_token_id, ckusdt_token_id);
    let pool1_icp = pool_map::get_by_token_ids(pay_token_id, icp_token_id);
    let pool2_icp = pool_map::get_by_token_ids(receive_token_id, icp_token_id);
    if pool1_ckusdt.is_some() && pool2_ckusdt.is_some() || pool1_icp.is_some() && pool2_icp.is_some() {
        let swaps_ckusdt = if pool1_ckusdt.is_some() && pool2_ckusdt.is_some() {
            let pool1 = pool1_ckusdt.clone().unwrap();
            let pool2 = pool2_ckusdt.clone().unwrap();
            // 2-step swap
            // split the lp fee between the two swaps. the "+ 1) / 2" will round up the integer
            // 1st swap no gas fees as this is intermediate swap
            // 2nd swap use standard gas fees
            let swap1_lp_fee = (pool1.lp_fee_bps + 1) / 2;
            let swap1 = swap_amount_0(&pool1, &pay_amount, Some(user_fee_level), Some(swap1_lp_fee), Some(&nat_zero()))?;
            let swap2_lp_fee = (pool2.lp_fee_bps + 1) / 2;
            let swap2 = swap_amount_1(
                &pool2,
                &swap1.receive_amount_with_fees_and_gas(),
                Some(user_fee_level),
                Some(swap2_lp_fee),
                None,
            )?;
            Some((swap1, swap2))
        } else {
            None
        };

        let swaps_icp = if pool1_icp.is_some() && pool2_icp.is_some() {
            let pool1 = pool1_icp.clone().unwrap();
            let pool2 = pool2_icp.clone().unwrap();
            let swap1_lp_fee = (pool1.lp_fee_bps + 1) / 2;
            let swap1 = swap_amount_0(&pool1, &pay_amount, Some(user_fee_level), Some(swap1_lp_fee), Some(&nat_zero()))?;
            let swap2_lp_fee = (pool2.lp_fee_bps + 1) / 2;
            let swap2 = swap_amount_1(
                &pool2,
                &swap1.receive_amount_with_fees_and_gas(),
                Some(user_fee_level),
                Some(swap2_lp_fee),
                None,
            )?;
            Some((swap1, swap2))
        } else {
            None
        };

        // if both ckUSDT and ICP are possible, choose the one with the highest receive amount
        let (swap1, swap2) = if swaps_ckusdt.is_some() && swaps_icp.is_some() {
            let swaps_ckusdt = swaps_ckusdt.unwrap();
            let swaps_icp = swaps_icp.unwrap();
            if swaps_ckusdt.1.receive_amount >= swaps_icp.1.receive_amount {
                (swaps_ckusdt.0, swaps_ckusdt.1)
            } else {
                (swaps_icp.0, swaps_icp.1)
            }
        } else if swaps_ckusdt.is_some() {
            let swaps_ckusdt = swaps_ckusdt.unwrap();
            (swaps_ckusdt.0, swaps_ckusdt.1)
        } else if swaps_icp.is_some() {
            let swaps_icp = swaps_icp.unwrap();
            (swaps_icp.0, swaps_icp.1)
        } else {
            // should never happen
            return Err("Pool not found".to_string());
        };
        txs.push(to_swap_amounts_tx_reply(&swap1).ok_or("Invalid swap tokens")?);
        txs.push(to_swap_amounts_tx_reply(&swap2).ok_or("Invalid swap tokens")?);
        let receive_amount = swap2.receive_amount_with_fees_and_gas();
        let swap1_price = swap1.get_price().ok_or("Invalid swap1 price")?;
        let swap2_price = swap2.get_price().ok_or("Invalid swap2 price")?;
        let price = swap1_price * swap2_price;
        let price_f64 = price_rounded(&price).ok_or("Invalid price")?;
        let swap1_mid_price = swap1.get_mid_price().ok_or("Invalid swap1 mid price")?;
        let swap2_mid_price = swap2.get_mid_price().ok_or("Invalid swap2 mid price")?;
        let mid_price = swap1_mid_price * swap2_mid_price;
        let mid_price_f64 = price_rounded(&mid_price).ok_or("Invalid mid price")?;
        let slippage_f64 = get_slippage(&price, &mid_price).ok_or("Invalid slippage")?;
        return Ok(SwapAmountsReply {
            pay_chain,
            pay_symbol,
            pay_address,
            pay_amount,
            receive_chain,
            receive_symbol,
            receive_address,
            receive_amount,
            price: price_f64,
            txs,
            mid_price: mid_price_f64,
            slippage: slippage_f64,
        });
    }

    // special case where pay token is ckUSDT and token0/ckUSDT pool does not exist so need to use token0/ICP pool
    let pool1_icp_ckusdt = pool_map::get_by_token_ids(icp_token_id, ckusdt_token_id);
    if pay_token_id == ckusdt_token_id && pool1_icp_ckusdt.is_some() && pool2_icp.is_some() {
        let pool1 = pool1_icp_ckusdt.clone().unwrap();
        let pool2 = pool2_icp.clone().unwrap();
        let swap1_lp_fee = (pool1.lp_fee_bps + 1) / 2;
        let swap1 = swap_amount_1(&pool1, &pay_amount, Some(user_fee_level), Some(swap1_lp_fee), Some(&nat_zero()))?;
        let swap2_lp_fee = (pool2.lp_fee_bps + 1) / 2;
        let swap2 = swap_amount_1(
            &pool2,
            &swap1.receive_amount_with_fees_and_gas(),
            Some(user_fee_level),
            Some(swap2_lp_fee),
            None,
        )?;
        txs.push(to_swap_amounts_tx_reply(&swap1).ok_or("Invalid swap tokens")?);
        txs.push(to_swap_amounts_tx_reply(&swap2).ok_or("Invalid swap tokens")?);
        let receive_amount = swap2.receive_amount_with_fees_and_gas();
        let swap1_price = swap1.get_price().ok_or("Invalid swap1 price")?;
        let swap2_price = swap2.get_price().ok_or("Invalid swap2 price")?;
        let price = swap1_price * swap2_price;
        let price_f64 = price_rounded(&price).ok_or("Invalid price")?;
        let swap1_mid_price = swap1.get_mid_price().ok_or("Invalid swap1 mid price")?;
        let swap2_mid_price = swap2.get_mid_price().ok_or("Invalid swap2 mid price")?;
        let mid_price = swap1_mid_price * swap2_mid_price;
        let mid_price_f64 = price_rounded(&mid_price).ok_or("Invalid mid price")?;
        let slippage_f64 = get_slippage(&price, &mid_price).ok_or("Invalid slippage")?;
        return Ok(SwapAmountsReply {
            pay_chain,
            pay_symbol,
            pay_address,
            pay_amount,
            receive_chain,
            receive_symbol,
            receive_address,
            receive_amount,
            price: price_f64,
            txs,
            mid_price: mid_price_f64,
            slippage: slippage_f64,
        });
    };

    // special case where receieve token is ckUSDT and token0/ckUSDT pool does not exist so need to use token0/ICP pool
    let pool2_icp_ckusdt = pool_map::get_by_token_ids(icp_token_id, ckusdt_token_id);
    if receive_token_id == ckusdt_token_id && pool1_icp.is_some() && pool2_icp_ckusdt.is_some() {
        let pool1 = pool1_icp.clone().unwrap();
        let pool2 = pool2_icp_ckusdt.clone().unwrap();
        let swap1_lp_fee = (pool1.lp_fee_bps + 1) / 2;
        let swap1 = swap_amount_0(&pool1, &pay_amount, Some(user_fee_level), Some(swap1_lp_fee), Some(&nat_zero()))?;
        let swap2_lp_fee = (pool2.lp_fee_bps + 1) / 2;
        let swap2 = swap_amount_0(
            &pool2,
            &swap1.receive_amount_with_fees_and_gas(),
            Some(user_fee_level),
            Some(swap2_lp_fee),
            None,
        )?;
        txs.push(to_swap_amounts_tx_reply(&swap1).ok_or("Invalid swap tokens")?);
        txs.push(to_swap_amounts_tx_reply(&swap2).ok_or("Invalid swap tokens")?);
        let receive_amount = swap2.receive_amount_with_fees_and_gas();
        let swap1_price = swap1.get_price().ok_or("Invalid swap1 price")?;
        let swap2_price = swap2.get_price().ok_or("Invalid swap2 price")?;
        let price = swap1_price * swap2_price;
        let price_f64 = price_rounded(&price).ok_or("Invalid price")?;
        let swap1_mid_price = swap1.get_mid_price().ok_or("Invalid swap1 mid price")?;
        let swap2_mid_price = swap2.get_mid_price().ok_or("Invalid swap2 mid price")?;
        let mid_price = swap1_mid_price * swap2_mid_price;
        let mid_price_f64 = price_rounded(&mid_price).ok_or("Invalid mid price")?;
        let slippage_f64 = get_slippage(&price, &mid_price).ok_or("Invalid slippage")?;
        return Ok(SwapAmountsReply {
            pay_chain,
            pay_symbol,
            pay_address,
            pay_amount,
            receive_chain,
            receive_symbol,
            receive_address,
            receive_amount,
            price: price_f64,
            txs,
            mid_price: mid_price_f64,
            slippage: slippage_f64,
        });
    };

    // test for 3-step swap via ckUSDT and then ICP
    let pool3_icp = pool2_icp;
    if pool1_ckusdt.is_some() && pool2_icp_ckusdt.is_some() && pool3_icp.is_some() {
        // token0 -> ckUSDT -> ICP -> token1
        let (swap1, swap2, swap3, price_f64, mid_price_f64, slippage_f64) = perform_3step_swap_amount_0(
            &pool1_ckusdt.unwrap(),
            &pool2_icp_ckusdt.unwrap(),
            &pool3_icp.unwrap(),
            &pay_amount,
            user_fee_level,
        )?;
        txs.push(to_swap_amounts_tx_reply(&swap1).ok_or("Invalid swap tokens")?);
        txs.push(to_swap_amounts_tx_reply(&swap2).ok_or("Invalid swap tokens")?);
        txs.push(to_swap_amounts_tx_reply(&swap3).ok_or("Invalid swap tokens")?);
        let receive_amount = swap3.receive_amount_with_fees_and_gas();
        return Ok(SwapAmountsReply {
            pay_chain,
            pay_symbol,
            pay_address,
            pay_amount,
            receive_chain,
            receive_symbol,
            receive_address,
            receive_amount,
            price: price_f64,
            txs,
            mid_price: mid_price_f64,
            slippage: slippage_f64,
        });
    };

    // test for 3-step swap via ICP and then ckUSDT
    let pool3_ckusdt = pool2_ckusdt;
    if pool1_icp.is_some() && pool2_icp_ckusdt.is_some() && pool3_ckusdt.is_some() {
        // token0 -> ICP -> ckUSDT -> token1
        let (swap1, swap2, swap3, price_f64, mid_price_f64, slippage_f64) = perform_3step_swap_amount_1(
            &pool1_icp.unwrap(),
            &pool2_icp_ckusdt.unwrap(),
            &pool3_ckusdt.unwrap(),
            &pay_amount,
            user_fee_level,
        )?;
        txs.push(to_swap_amounts_tx_reply(&swap1).ok_or("Invalid swap tokens")?);
        txs.push(to_swap_amounts_tx_reply(&swap2).ok_or("Invalid swap tokens")?);
        txs.push(to_swap_amounts_tx_reply(&swap3).ok_or("Invalid swap tokens")?);
        let receive_amount = swap3.receive_amount_with_fees_and_gas();
        return Ok(SwapAmountsReply {
            pay_chain,
            pay_symbol,
            pay_address,
            pay_amount,
            receive_chain,
            receive_symbol,
            receive_address,
            receive_amount,
            price: price_f64,
            txs,
            mid_price: mid_price_f64,
            slippage: slippage_f64,
        });
    };

    Err("Pool not found".to_string())
}

fn perform_3step_swap_amount_0(
    pool1: &StablePool,
    pool2: &StablePool,
    pool3: &StablePool,
    pay_amount: &Nat,
    user_fee_level: u8,
) -> Result<(SwapCalc, SwapCalc, SwapCalc, f64, f64, f64), String> {
    let swap1_lp_fee = (pool1.lp_fee_bps + 1) / 3; // this will round it up
    let swap1 = swap_amount_0(
        pool1,
        pay_amount,
        Some(user_fee_level),
        Some(swap1_lp_fee),
        Some(&nat_zero()), // swap1 do not take gas fees
    )?;
    let swap2_lp_fee = (pool2.lp_fee_bps + 1) / 3;
    let swap2 = swap_amount_1(
        pool2,
        &swap1.receive_amount_with_fees_and_gas(),
        Some(user_fee_level),
        Some(swap2_lp_fee),
        Some(&nat_zero()), // swap2 do not take gas fees
    )?;
    let swap3_lp_fee = (pool3.lp_fee_bps + 1) / 3;
    let swap3 = swap_amount_1(
        pool3,
        &swap2.receive_amount_with_fees_and_gas(),
        Some(user_fee_level),
        Some(swap3_lp_fee),
        None,
    )?;
    let swap1_price = swap1.get_price().ok_or("Invalid swap1 price")?;
    let swap2_price = swap2.get_price().ok_or("Invalid swap2 price")?;
    let swap3_price = swap3.get_price().ok_or("Invalid swap3 price")?;
    let price = swap1_price * swap2_price * swap3_price;
    let price_f64 = price_rounded(&price).ok_or("Invalid price")?;
    let swap1_mid_price = swap1.get_mid_price().ok_or("Invalid swap1 mid price")?;
    let swap2_mid_price = swap2.get_mid_price().ok_or("Invalid swap2 mid price")?;
    let swap3_mid_price = swap3.get_mid_price().ok_or("Invalid swap3 mid price")?;
    let mid_price = swap1_mid_price * swap2_mid_price * swap3_mid_price;
    let mid_price_f64 = price_rounded(&mid_price).ok_or("Invalid mid price")?;
    let slippage_f64 = get_slippage(&price, &mid_price).ok_or("Invalid slippage")?;
    Ok((swap1, swap2, swap3, price_f64, mid_price_f64, slippage_f64))
}

fn perform_3step_swap_amount_1(
    pool1: &StablePool,
    pool2: &StablePool,
    pool3: &StablePool,
    pay_amount: &Nat,
    user_fee_level: u8,
) -> Result<(SwapCalc, SwapCalc, SwapCalc, f64, f64, f64), String> {
    let swap1_lp_fee = (pool1.lp_fee_bps + 1) / 3; // this will round it up
    let swap1 = swap_amount_0(
        pool1,
        pay_amount,
        Some(user_fee_level),
        Some(swap1_lp_fee),
        Some(&nat_zero()), // swap1 do not take gas fees
    )?;
    let swap2_lp_fee = (pool2.lp_fee_bps + 1) / 3;
    let swap2 = swap_amount_0(
        pool2,
        &swap1.receive_amount_with_fees_and_gas(),
        Some(user_fee_level),
        Some(swap2_lp_fee),
        Some(&nat_zero()), // swap2 do not take gas fees
    )?;
    let swap3_lp_fee = (pool3.lp_fee_bps + 1) / 3;
    let swap3 = swap_amount_1(
        pool3,
        &swap2.receive_amount_with_fees_and_gas(),
        Some(user_fee_level),
        Some(swap3_lp_fee),
        None,
    )?;
    let swap1_price = swap1.get_price().ok_or("Invalid swap1 price")?;
    let swap2_price = swap2.get_price().ok_or("Invalid swap2 price")?;
    let swap3_price = swap3.get_price().ok_or("Invalid swap3 price")?;
    let price = swap1_price * swap2_price * swap3_price;
    let price_f64 = price_rounded(&price).ok_or("Invalid price")?;
    let swap1_mid_price = swap1.get_mid_price().ok_or("Invalid swap1 mid price")?;
    let swap2_mid_price = swap2.get_mid_price().ok_or("Invalid swap2 mid price")?;
    let swap3_mid_price = swap3.get_mid_price().ok_or("Invalid swap3 mid price")?;
    let mid_price = swap1_mid_price * swap2_mid_price * swap3_mid_price;
    let mid_price_f64 = price_rounded(&mid_price).ok_or("Invalid mid price")?;
    let slippage_f64 = get_slippage(&price, &mid_price).ok_or("Invalid slippage")?;
    Ok((swap1, swap2, swap3, price_f64, mid_price_f64, slippage_f64))
}
