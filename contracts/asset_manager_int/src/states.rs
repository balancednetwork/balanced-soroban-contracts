use soroban_sdk::{Address, Env, String, Vec};

use crate::{errors::ContractError, storage_types::{DataKey, TokenData}};

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

pub fn read_token_data(env: &Env, token_address: Address) -> Result<TokenData, ContractError> {
    let key = DataKey::TokenData(token_address);
    let token_data: TokenData = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::TokenDoesNotExists)?;
    Ok(token_data)
}

pub fn write_tokens(e: &Env, token: Address) {
    let key = DataKey::Tokens;
    let mut tokens: Vec<Address> = match e.storage().persistent().get(&key) {
        Some(names) => names,
        None => Vec::new(&e),
    };

    tokens.push_back(token);
    e.storage().persistent().set(&key, &tokens);
}

pub fn read_tokens(e: &Env) -> Vec<Address> {
    let key = DataKey::Tokens;
    let tokens: Vec<Address> = match e.storage().persistent().get(&key) {
        Some(names) => names,
        None => Vec::new(&e),
    };

    tokens
}

pub fn extent_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    let tokens = read_tokens(&e);
    e.storage().persistent().extend_ttl(
        &DataKey::Tokens,
        INSTANCE_LIFETIME_THRESHOLD,
        INSTANCE_BUMP_AMOUNT,
    );
    for token in tokens {

        e.storage().persistent().extend_ttl(
            &DataKey::TokenData(token.clone()),
            INSTANCE_LIFETIME_THRESHOLD,
            INSTANCE_BUMP_AMOUNT,
        );

    }
}
