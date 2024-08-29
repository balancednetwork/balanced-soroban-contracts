use soroban_sdk::{Address, Env, String, Vec};

use crate::storage_types::DataKey;

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub fn read_administrator(e: &Env) -> Address {
    let key = DataKey::Admin;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    let key = DataKey::Admin;
    e.storage().instance().set(&key, id);
}

pub fn has_registry(env:Env) -> bool {
    env.storage().instance().has(&DataKey::Registry)
}

pub fn write_registry(e: &Env, id: &Address) {
    let key = DataKey::Registry;
    e.storage().instance().set(&key, id);
} 

pub fn has_proposed_removed(env:Env) -> bool {
    env.storage().instance().has(&DataKey::ProposedProtocolToRemove)
}

pub fn write_proposed_removed(e: &Env, id: &String) {
    let key = DataKey::ProposedProtocolToRemove;
    e.storage().instance().set(&key, id);
}

pub fn read_proposed_removed(e: &Env) -> String {
    let key = DataKey::ProposedProtocolToRemove;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_sources(e: &Env, id: &Vec<String>) {
    let key = DataKey::Sources;
    e.storage().instance().set(&key, id);
}

pub fn read_sources(e: &Env) -> Vec<String> {
    let key = DataKey::Sources;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_destinations(e: &Env, id: &Vec<String>) {
    let key = DataKey::Destinations;
    e.storage().instance().set(&key, id);
}

pub fn read_destinations(e: &Env) -> Vec<String> {
    let key = DataKey::Destinations;
    e.storage().instance().get(&key).unwrap()
}

pub fn extend_ttl (e: &Env){
    e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}