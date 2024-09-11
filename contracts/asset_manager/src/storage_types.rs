use soroban_sdk::{contracttype, Address};

pub(crate) const POINTS: u128 = 10000;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Registry,
    Admin,
    Config,
    Tokens,
    TokenData(Address)
}

#[derive(Clone)]
#[contracttype]
pub struct TokenData {
    pub period: u64,
    pub percentage: u32,
    pub last_update: u64,
    pub current_limit: u64,
}
