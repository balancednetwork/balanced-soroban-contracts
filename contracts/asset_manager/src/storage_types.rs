use soroban_sdk::{contracttype, Address, String};

pub(crate) const POINTS: u128 = 10000;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    XCall,
    XcallManager,
    NativeAddress,
    IconAssetManager,
    UpgradeAuthority,
    Period(Address),
    Percentage(Address),
    LastUpdate(Address),
    CurrentLimit(Address),
}

#[contracttype]
pub struct ConfigData {
    pub xcall: Address,
    pub xcall_manager: Address,
    pub native_address: Address,
    pub icon_asset_manager: String,
    pub upgrade_authority: Address,
}

#[contracttype]
pub struct RateLimit {
    pub period: u64,
    pub percentage: u32,
    pub last_update: u64,
    pub current_limit: u64,
}

