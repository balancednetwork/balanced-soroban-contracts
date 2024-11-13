use soroban_sdk::{Address, Env, String};

use crate::storage_types::DataKey;

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub fn has_administrator(e: &Env) -> bool {
    let key: DataKey = DataKey::Admin;
    e.storage().instance().has(&key)
}

pub fn read_administrator(e: &Env) -> Address {
    let key = DataKey::Admin;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    let key = DataKey::Admin;
    e.storage().instance().set(&key, id);
}

pub fn has_upgrade_authority(e: &Env) -> bool {
    let key = DataKey::UpgradeAuthority;
    e.storage().instance().has(&key)
}

pub fn write_xcall(env: &Env, xcall: Address) {
    let key = DataKey::XCall;
    env.storage().instance().set(&key, &xcall);
}

pub fn write_xcall_manager(env: &Env, xcall_manager: Address) {
    let key = DataKey::XcallManager;
    env.storage().instance().set(&key, &xcall_manager);
}

pub fn write_native_address(env: &Env, native_address: Address) {
    let key = DataKey::NativeAddress;
    env.storage().instance().set(&key, &native_address);
}

pub fn write_icon_asset_manager(env: &Env, icon_asset_manager: String) {
    let key = DataKey::IconAssetManager;
    env.storage().instance().set(&key, &icon_asset_manager);
}

pub fn write_upgrade_authority(env: &Env, upgrade_authority: Address) {
    let key = DataKey::UpgradeAuthority;
    env.storage().instance().set(&key, &upgrade_authority);
}

pub fn read_upgrade_authority(env: &Env) -> Address {
    let key = DataKey::UpgradeAuthority;
    env.storage().instance().get(&key).unwrap()
}

pub fn read_native_address(env: &Env) -> Address {
    let key = DataKey::NativeAddress;
    env.storage().instance().get(&key).unwrap()
}

pub fn read_icon_asset_manager(env: &Env) -> String {
    let key = DataKey::IconAssetManager;
    env.storage().instance().get(&key).unwrap()
}

pub fn read_xcall_manager(env: &Env) -> Address {
    let key = DataKey::XcallManager;
    env.storage().instance().get(&key).unwrap()
}

pub fn read_xcall(env: &Env) -> Address {
    let key = DataKey::XCall;
    env.storage().instance().get(&key).unwrap()
}

pub fn write_period(env: &Env, token_address: Address, period: u64) {
    let key = DataKey::Period(token_address);
    env.storage().instance().set(&key, &period);
}

pub fn read_period(env: &Env, token_address: Address) -> u64 {
    let key = DataKey::Period(token_address);
    env.storage().instance().get(&key).unwrap()
}

pub fn write_percentage(env: &Env, token_address: Address, percentage: u32) {
    let key = DataKey::Percentage(token_address);
    env.storage().instance().set(&key, &percentage);
}

pub fn read_percentage(env: &Env, token_address: Address) -> u32 {
    let key = DataKey::Percentage(token_address);
    env.storage().instance().get(&key).unwrap()
}

pub fn write_last_update(env: &Env, token_address: Address, last_update: u64) {
    let key = DataKey::LastUpdate(token_address);
    env.storage().instance().set(&key, &last_update);
}

pub fn read_last_update(env: &Env, token_address: Address) -> u64 {
    let key = DataKey::LastUpdate(token_address);
    env.storage().instance().get(&key).unwrap()
}

pub fn write_current_limit(env: &Env, token_address: Address, current_limit: u64) {
    let key = DataKey::CurrentLimit(token_address);
    env.storage().instance().set(&key, &current_limit);
}

pub fn read_current_limit(env: &Env, token_address: Address) -> u64 {
    let key = DataKey::CurrentLimit(token_address);
    env.storage().instance().get(&key).unwrap()
}

pub fn extent_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}
