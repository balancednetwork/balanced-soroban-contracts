use soroban_sdk::{Address, Env, String, Vec};

use crate::storage_types::DataKey;

pub fn has_state(env:Env, key: DataKey) -> bool {
    env.storage().instance().has(&key)
}

pub fn write_string_state(e: &Env, key: DataKey, id: &String) {
    e.storage().instance().set(&key, id);
}

pub fn write_address_state(e: &Env, key: DataKey, id: &Address) {
    e.storage().instance().set(&key, id);
}

pub fn read_address_state(e: &Env, key: DataKey) -> Address {
    e.storage().instance().get(&key).unwrap()
}

pub fn write_vec_string_state(e: &Env, key: DataKey, id: &Vec<String>) {
    e.storage().instance().set(&key, id);
}

pub fn read_string_state(e: &Env, key: DataKey) -> String {
    e.storage().instance().get(&key).unwrap()
}

pub fn read_vec_string_state(e: &Env, key: DataKey) -> Vec<String> {
    e.storage().instance().get(&key).unwrap()
}