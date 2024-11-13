use soroban_sdk::{contracttype, Address};

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
pub struct ConfigData {
    pub xcall: Address,
    pub xcall_manager: Address,
    pub nid: String,
    pub icon_bn_usd: String,
    pub upgrade_authority: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Allowance(AllowanceDataKey),
    Balance(Address),
    Admin,
    XcallManager,
    XCall,
    Nid,
    IconBnusd,
    UpgradeAuthority
}

pub fn set_xcall_manager(e: &Env, value: XcallManager) {
    e.storage().instance().set(&DataKey::XcallManager, &value);
}

pub fn set_xcall(e: &Env, value: Xcall) {
    e.storage().instance().set(&DataKey::Xcall, &value);
}

pub fn set_nid(e: &Env, value: Nid) {
    e.storage().instance().set(&DataKey::Nid, &value);
}

pub fn set_icon_bnusd(e: &Env, value: IconBnusd) {
    e.storage().instance().set(&DataKey::IconBnusd, &value);
}

pub fn set_upgrade_authority(e: &Env, value: UpgradeAuthority) {
    e.storage().instance().set(&DataKey::UpgradeAuthority, &value);
}


pub fn get_xcall_manager(e: &Env) -> XcallManager {
    let key = DataKey::XcallManager;
    e.storage().instance().get(&key).unwrap_optimized()
}

pub fn get_xcall(e: &Env) -> Xcall {
    let key = DataKey::Xcall;
    e.storage().instance().get(&key).unwrap_optimized()
}

pub fn get_nid(e: &Env) -> Nid {
    let key = DataKey::Nid;
    e.storage().instance().get(&key).unwrap_optimized()
}

pub fn get_icon_bnusd(e: &Env) -> IconBnusd {
    let key = DataKey::IconBnusd;
    e.storage().instance().get(&key).unwrap_optimized()
}

pub fn get_upgrade_authority(e: &Env) -> UpgradeAuthority {
    let key = DataKey::UpgradeAuthority;
    e.storage().instance().get(&key).unwrap_optimized()
}




