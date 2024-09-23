use crate::storage_types::DataKey;
use soroban_sdk::{contracttype, unwrap::UnwrapOptimized, Address, Env, String};

#[derive(Clone)]
#[contracttype]
pub struct ConfigData {
    pub xcall: Address,
    pub xcall_manager: Address,
    pub native_address: Address,
    pub icon_asset_manager: String,
    pub xcall_network_address: String,
    pub upgrade_authority: Address,
}

pub fn set_config(e: &Env, config: ConfigData) {
    e.storage().instance().set(&DataKey::Config, &config);
}

pub fn get_config(e: &Env) -> ConfigData {
    let key = DataKey::Config;
    e.storage().instance().get(&key).unwrap_optimized()
}
