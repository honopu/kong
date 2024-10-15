use candid::{CandidType, Principal};
use ic_ledger_types::AccountIdentifier;
use icrc_ledger_types::icrc1::account::Account;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static PRINCIPAL_ID_LOCK: OnceLock<Regex> = OnceLock::new();
const PRINCIPAL_ID_REGEX: &str = r"^([a-z0-9]{5}-){10}[a-z0-9]{3}|([a-z0-9]{5}-){4}cai$";
static ACCOUNT_ID_LOCK: OnceLock<Regex> = OnceLock::new();
const ACCOUNT_ID_REGEX: &str = r"^[a-f0-9]{64}$";

/// Represents an address which can be either an Account ID or a Principal ID.
#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub enum Address {
    AccountId(AccountIdentifier),
    PrincipalId(Account),
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::AccountId(account_id) => write!(f, "{}", account_id),
            Address::PrincipalId(principal_id) => write!(f, "{}", principal_id),
        }
    }
}

pub fn is_principal_id(address: &str) -> bool {
    let regrex_principal_id = PRINCIPAL_ID_LOCK.get_or_init(|| Regex::new(PRINCIPAL_ID_REGEX).unwrap());
    regrex_principal_id.is_match(address)
}

pub fn get_address(address: &str) -> Option<Address> {
    let regrex_princiapl_id = PRINCIPAL_ID_LOCK.get_or_init(|| Regex::new(PRINCIPAL_ID_REGEX).unwrap());
    let regrex_account_id = ACCOUNT_ID_LOCK.get_or_init(|| Regex::new(ACCOUNT_ID_REGEX).unwrap());

    if regrex_princiapl_id.is_match(address) {
        Some(super::address::Address::PrincipalId(Account::from(
            Principal::from_text(address).ok()?,
        )))
    } else if regrex_account_id.is_match(address) {
        Some(super::address::Address::AccountId(
            AccountIdentifier::from_hex(address).ok()?,
        ))
    } else {
        None
    }
}
