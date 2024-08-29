use soroban_sdk::{contracttype, Address};

pub(crate) const POINTS: u128 = 10000;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Registry,
    Admin,
    Config,
    Tokens,
    Period(Address),
    Percentage(Address),
    LastUpdate(Address),
    CurrentLimit(Address),
}
