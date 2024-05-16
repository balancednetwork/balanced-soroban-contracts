use soroban_sdk::{contracttype, unwrap::UnwrapOptimized, Env, String, Address};
use crate::storage_types::DataKey;

#[derive(Clone)]
#[contracttype]
pub struct ConfigData {
    pub xcall: Address,
    pub icon_governance: String,
    pub xcall_network_address: String,
}

pub fn get_config(e: &Env) -> ConfigData {
    let key = DataKey::Config;
    e
    .storage()
    .instance()
    .get(&key)
    .unwrap_optimized()
}

