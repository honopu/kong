use kong_lib::ic::id::{caller_principal_id, principal_id_is_not_anonymous};
use kong_lib::stable_user::stable_user::StableUser;

use crate::stable_memory::USER_MAP;

/// return StableUser by principal_id
///
/// # Arguments
///
/// principal_id - principal_id of the user
///
/// # Returns
///
/// * `Ok(Some(StableUser))` if user is not anonymous, if principal_id is a known user or None if user is not registered
/// * `Err(String)` if user is anonymous
pub fn get_by_principal_id(principal_id: &str) -> Result<Option<StableUser>, String> {
    principal_id_is_not_anonymous(principal_id)?;

    Ok(USER_MAP.with(|m| {
        m.borrow()
            .iter()
            .find_map(|(_, v)| if v.principal_id == principal_id { Some(v) } else { None })
    }))
}

/// return StableUser of the caller
///
/// # Returns
///
/// * `Ok(Some(StableUser))` if user is not anonymous, if principal_id is a known user or None if user is not registered
/// * `Err(String)` if user is anonymous
pub fn get_by_caller() -> Result<Option<StableUser>, String> {
    get_by_principal_id(&caller_principal_id())
}
