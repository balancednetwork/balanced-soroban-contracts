use soroban_sdk::{contracttype, Address};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const  POINTS: i128 = 1000;

#[derive(Clone)]
#[contracttype]
pub enum DataKey{
    Registry,
    Admin,
    XCall,
    XCallNetworkAddress,
    IconAssetManager,
    XCallManager,
    NativeAddress,
    Period(Address),
    Percentage(Address),
    LastUpdate(Address),
    CurrentLimit(Address)
}