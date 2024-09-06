use crate::storage_types::DataKey;
use soroban_sdk::{contracttype, unwrap::UnwrapOptimized, Address, Env, String};

#[derive(Clone)]
#[contracttype]
pub struct ConfigData {
    pub xcall: Address,
    pub xcall_manager: Address,
    pub nid: String,
    pub icon_bn_usd: String,
}

pub fn set_config(e: &Env, config: ConfigData) {
    e.storage().instance().set(&DataKey::Config, &config);
}

pub fn get_config(e: &Env) -> ConfigData {
    let key = DataKey::Config;
    e.storage().instance().get(&key).unwrap_optimized()
}
