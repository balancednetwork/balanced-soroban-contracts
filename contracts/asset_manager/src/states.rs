use soroban_sdk::{Address, Env, Vec, String};

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

pub fn write_token_data(env: &Env, token_address: Address, data: TokenData) {
    let key = DataKey::TokenData(token_address);
    env.storage().persistent().set(&key, &data);
}

pub fn set_xcall_network_address(e: &Env, value: String) {
    e.storage().instance().set(&DataKey::XCallNetworkAddress, &value);
}

pub fn get_xcall_network_address(e: &Env) -> Result<String, ContractError> {
    let key = DataKey::XCallNetworkAddress;
    e.storage()
        .instance()
        .get(&key)
        .ok_or(ContractError::Uninitialized)
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
