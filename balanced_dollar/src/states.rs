use soroban_sdk::{Address, Env};

use crate::storage_types::DataKey;

pub fn has_state(env:Env, key: DataKey) -> bool {
    env.storage().instance().has(&key)
}

