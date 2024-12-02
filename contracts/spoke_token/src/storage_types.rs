use soroban_sdk::{contracttype, Address, Env, String};

use crate::errors::ContractError;

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[derive(Clone)]
#[contracttype]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Allowance(AllowanceDataKey),
    Balance(Address),
    XcallManager,
    XCall,
    HubToken,
    UpgradeAuthority,
    XCallNetworkAddress,
}

pub fn set_xcall_manager(e: &Env, value: Address) {
    e.storage().instance().set(&DataKey::XcallManager, &value);
}

pub fn set_xcall(e: &Env, value: Address) {
    e.storage().instance().set(&DataKey::XCall, &value);
}

pub fn set_icon_hub_token(e: &Env, value: String) {
    e.storage().instance().set(&DataKey::HubToken, &value);
}

pub fn set_upgrade_authority(e: &Env, value: Address) {
    e.storage().instance().set(&DataKey::UpgradeAuthority, &value);
}

pub fn set_xcall_network_address(e: &Env, value: String) {
    e.storage().instance().set(&DataKey::XCallNetworkAddress, &value);
}

pub fn has_upgrade_auth(e: &Env) -> bool {
    let key = DataKey::UpgradeAuthority;
    e.storage().instance().has(&key)
}

pub fn get_xcall_manager(e: &Env) -> Result<Address, ContractError> {
    let key = DataKey::XcallManager;
    e.storage()
        .instance()
        .get(&key)
        .ok_or(ContractError::Uninitialized)}

pub fn get_xcall(e: &Env) -> Result<Address, ContractError> {
    let key = DataKey::XCall;
    e.storage()
        .instance()
        .get(&key)
        .ok_or(ContractError::Uninitialized)}

pub fn get_icon_hub_token(e: &Env) -> Result<String, ContractError> {
    let key = DataKey::HubToken;
    e.storage()
        .instance()
        .get(&key)
        .ok_or(ContractError::Uninitialized)}

pub fn get_upgrade_authority(e: &Env) -> Result<Address, ContractError> {
    let key = DataKey::UpgradeAuthority;
    e.storage()
        .instance()
        .get(&key)
        .ok_or(ContractError::Uninitialized)}

pub fn get_xcall_network_address(e: &Env) -> Result<String, ContractError> {
    let key = DataKey::XCallNetworkAddress;
    e.storage()
        .instance()
        .get(&key)
        .ok_or(ContractError::Uninitialized)}




