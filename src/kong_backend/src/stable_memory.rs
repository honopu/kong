use ic_cdk_timers::TimerId;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use std::cell::{Cell, RefCell};

use crate::stable_claim::stable_claim::{StableClaim, StableClaimId};
use crate::stable_kong_settings::stable_kong_settings::StableKongSettings;
use crate::stable_lp_token_ledger::stable_lp_token_ledger::{StableLPTokenLedger, StableLPTokenLedgerId};
use crate::stable_lp_token_ledger::stable_lp_token_ledger_old::{StableLPTokenLedgerIdOld, StableLPTokenLedgerOld};
use crate::stable_message::stable_message::{StableMessage, StableMessageId};
use crate::stable_pool::stable_pool::{StablePool, StablePoolId};
use crate::stable_request::stable_request::{StableRequest, StableRequestId};
use crate::stable_request::stable_request_old::{StableRequestIdOld, StableRequestOld};
use crate::stable_token::stable_token::{StableToken, StableTokenId};
use crate::stable_transfer::stable_transfer::{StableTransfer, StableTransferId};
use crate::stable_transfer::stable_transfer_old::{StableTransferIdOld, StableTransferOld};
use crate::stable_tx::stable_tx::{StableTx, StableTxId};
use crate::stable_user::stable_user::{StableUser, StableUserId};
use crate::stable_user::stable_user_old::{StableUserIdOld, StableUserOld};

type Memory = VirtualMemory<DefaultMemoryImpl>;

// old
pub const USER_OLD_MEMORY_ID: MemoryId = MemoryId::new(1);
pub const REQUEST_OLD_MEMORY_ID: MemoryId = MemoryId::new(5);
pub const TRANSFER_OLD_MEMORY_ID: MemoryId = MemoryId::new(6);
pub const LP_TOKEN_LEDGER_OLD_MEMORY_ID: MemoryId = MemoryId::new(8);
// stable memory
pub const KONG_SETTINGS_MEMORY_ID: MemoryId = MemoryId::new(20);
pub const USER_MEMORY_ID: MemoryId = MemoryId::new(21);
pub const TOKEN_MEMORY_ID: MemoryId = MemoryId::new(22);
pub const POOL_MEMORY_ID: MemoryId = MemoryId::new(23);
pub const TX_MEMORY_ID: MemoryId = MemoryId::new(24);
pub const TX_24H_MEMORY_ID: MemoryId = MemoryId::new(25);
pub const REQUEST_MEMORY_ID: MemoryId = MemoryId::new(26);
pub const TRANSFER_MEMORY_ID: MemoryId = MemoryId::new(27);
pub const CLAIM_MEMORY_ID: MemoryId = MemoryId::new(28);
pub const LP_TOKEN_LEDGER_MEMORY_ID: MemoryId = MemoryId::new(29);
pub const MESSAGE_MEMORY_ID: MemoryId = MemoryId::new(30);
// archives
pub const TX_ARCHIVE_MEMORY_ID: MemoryId = MemoryId::new(204);
pub const REQUEST_ARCHIVE_MEMORY_ID: MemoryId = MemoryId::new(205);
pub const TRANSFER_ARCHIVE_MEMORY_ID: MemoryId = MemoryId::new(206);

thread_local! {
    // static variable to store the timer id for the background claims timer
    // doesn't need to be in stable memory as they are not persisted across upgrades
    pub static CLAIMS_TIMER_ID: Cell<TimerId> = Cell::default();

    // static variable to store the timer id for the background stats timer
    pub static STATS_TIMER_ID: Cell<TimerId> = Cell::default();

    // static variable to store the timer id for the background tx map archive timer
    pub static TX_MAP_ARCHIVE_TIMER_ID: Cell<TimerId> = Cell::default();

    // static variable to store the timer id for the background request map archive timer
    pub static REQUEST_MAP_ARCHIVE_TIMER_ID: Cell<TimerId> = Cell::default();

    // static variable to store the timer id for the background transfer archive timer
    pub static TRANSFER_MAP_ARCHIVE_TIMER_ID: Cell<TimerId> = Cell::default();

    // MEMORY_MANAGER is given management of the entire stable memory. Given a 'MemoryId', it can
    // return a memory that can be used by stable structures
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // stable memory for storing user profiles
    pub static USER_OLD_MAP: RefCell<StableBTreeMap<StableUserIdOld, StableUserOld, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(USER_OLD_MEMORY_ID)))
    });

    // stable memory for storing all requests made by users
    pub static REQUEST_OLD_MAP: RefCell<StableBTreeMap<StableRequestIdOld, StableRequestOld, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(REQUEST_OLD_MEMORY_ID)))
    });

    // stable memory for storing all on-chain transfers with block_id. used to prevent accepting transfer twice (double receive)
    pub static TRANSFER_OLD_MAP: RefCell<StableBTreeMap<StableTransferIdOld, StableTransferOld, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(TRANSFER_OLD_MEMORY_ID)))
    });

    // stable memory for storing all LP tokens for users
    pub static LP_TOKEN_LEDGER_OLD: RefCell<StableBTreeMap<StableLPTokenLedgerIdOld, StableLPTokenLedgerOld, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(LP_TOKEN_LEDGER_OLD_MEMORY_ID)))
    });

    //
    // New Stable Memory
    //

    // stable memory for storing Kong settings
    pub static KONG_SETTINGS: RefCell<StableCell<StableKongSettings, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableCell::init(memory_manager.get(KONG_SETTINGS_MEMORY_ID), StableKongSettings::default()).expect("Failed to initialize Kong settings"))
    });

    // stable memory for storing user profiles
    pub static USER_MAP: RefCell<StableBTreeMap<StableUserId, StableUser, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(USER_MEMORY_ID)))
    });

    // stable memory for storing tokens supported by the system
    pub static TOKEN_MAP: RefCell<StableBTreeMap<StableTokenId, StableToken, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(TOKEN_MEMORY_ID)))
    });

    // stable memory for storing pools
    pub static POOL_MAP: RefCell<StableBTreeMap<StablePoolId, StablePool, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(POOL_MEMORY_ID)))
    });

    // stable memory for storing all transactions
    pub static TX_MAP: RefCell<StableBTreeMap<StableTxId, StableTx, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(TX_MEMORY_ID)))
    });

    // stable memory for storing txs for the last 24 hours. used for calculating rolling stats
    pub static TX_24H_MAP: RefCell<StableBTreeMap<StableTxId, StableTx, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(TX_24H_MEMORY_ID)))
    });

    // stable memory for storing all requests made by users
    pub static REQUEST_MAP: RefCell<StableBTreeMap<StableRequestId, StableRequest, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(REQUEST_MEMORY_ID)))
    });

    // stable memory for storing all on-chain transfers with block_id. used to prevent accepting transfer twice (double receive)
    pub static TRANSFER_MAP: RefCell<StableBTreeMap<StableTransferId, StableTransfer, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(TRANSFER_MEMORY_ID)))
    });

    // stable memory for storing all claims for users
    pub static CLAIM_MAP: RefCell<StableBTreeMap<StableClaimId, StableClaim, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(CLAIM_MEMORY_ID)))
    });

    // stable memory for storing all LP tokens for users
    pub static LP_TOKEN_LEDGER: RefCell<StableBTreeMap<StableLPTokenLedgerId, StableLPTokenLedger, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(LP_TOKEN_LEDGER_MEMORY_ID)))
    });

    // stable memory for storing all messages
    pub static MESSAGE_MAP: RefCell<StableBTreeMap<StableMessageId, StableMessage, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(MESSAGE_MEMORY_ID)))
    });

    //
    // Archive Stable Memory
    //

    // stable memory for storing tx archive
    pub static TX_ARCHIVE_MAP: RefCell<StableBTreeMap<StableTxId, StableTx, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(TX_ARCHIVE_MEMORY_ID)))
    });

    // stable memory for storing request archive
    pub static REQUEST_ARCHIVE_MAP: RefCell<StableBTreeMap<StableRequestId, StableRequest, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(REQUEST_ARCHIVE_MEMORY_ID)))
    });

    // stable memory for storing transfer archive
    pub static TRANSFER_ARCHIVE_MAP: RefCell<StableBTreeMap<StableTransferId, StableTransfer, Memory>> = with_memory_manager(|memory_manager| {
        RefCell::new(StableBTreeMap::init(memory_manager.get(TRANSFER_ARCHIVE_MEMORY_ID)))
    });
}

/// A helper function to access the memory manager.
pub fn with_memory_manager<R>(f: impl FnOnce(&MemoryManager<DefaultMemoryImpl>) -> R) -> R {
    MEMORY_MANAGER.with(|cell| f(&cell.borrow()))
}
