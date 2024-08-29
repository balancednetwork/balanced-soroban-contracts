use soroban_sdk::{Address, Env, Vec};

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

pub fn has_registry(e: &Env) -> bool {
    let key = DataKey::Registry;
    e.storage().instance().has(&key)
}

pub fn read_registry(e: &Env) -> Address {
    let key = DataKey::Registry;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_registry(e: &Env, id: &Address) {
    let key = DataKey::Registry;
    e.storage().instance().set(&key, id);
}

pub fn write_token_period(e: &Env, token: &Address, period: u128) {
    let key = DataKey::Period(token.clone());
    e.storage().persistent().set(&key, &period);
}

pub fn read_token_period(e: &Env, token: &Address) -> u128 {
    let key = DataKey::Period(token.clone());
    e.storage().persistent().get(&key).unwrap()
}

pub fn write_token_percentage(e: &Env, token: &Address, period: u128) {
    let key = DataKey::Percentage(token.clone());
    e.storage().persistent().set(&key, &period);
}

pub fn read_token_percentage(e: &Env, token: &Address) -> u128 {
    let key = DataKey::Percentage(token.clone());
    e.storage().persistent().get(&key).unwrap()
}

pub fn write_token_last_update(e: &Env, token: &Address, last_update: u64) {
    let key = DataKey::LastUpdate(token.clone());
    e.storage().persistent().set(&key, &last_update);
}

pub fn read_token_last_update(e: &Env, token: &Address) -> u64 {
    let key = DataKey::LastUpdate(token.clone());
    e.storage().persistent().get(&key).unwrap()
}

pub fn write_token_current_limit(e: &Env, token: &Address, current_limit: u128) {
    let key = DataKey::CurrentLimit(token.clone());
    e.storage().persistent().set(&key, &current_limit);
}

pub fn read_token_last_current_limit(e: &Env, token: &Address) -> u128 {
    let key = DataKey::CurrentLimit(token.clone());
    e.storage().persistent().get(&key).unwrap()
}

pub fn write_tokens(e: &Env, token: Address) {
    let key = DataKey::Tokens;
    let mut tokens: Vec<Address> = match e.storage().instance().get(&key) {
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

    let tokens = read_tokens(&e.clone());
    e.storage().persistent().extend_ttl(
        &DataKey::Tokens,
        INSTANCE_LIFETIME_THRESHOLD,
        INSTANCE_BUMP_AMOUNT,
    );
    for token in tokens {
        e.storage().persistent().extend_ttl(
            &DataKey::Period(token.clone()),
            INSTANCE_LIFETIME_THRESHOLD,
            INSTANCE_BUMP_AMOUNT,
        );
        e.storage().persistent().extend_ttl(
            &DataKey::Percentage(token.clone()),
            INSTANCE_LIFETIME_THRESHOLD,
            INSTANCE_BUMP_AMOUNT,
        );
        e.storage().persistent().extend_ttl(
            &DataKey::LastUpdate(token.clone()),
            INSTANCE_LIFETIME_THRESHOLD,
            INSTANCE_BUMP_AMOUNT,
        );
        e.storage().persistent().extend_ttl(
            &DataKey::CurrentLimit(token.clone()),
            INSTANCE_LIFETIME_THRESHOLD,
            INSTANCE_BUMP_AMOUNT,
        );
    }
}
