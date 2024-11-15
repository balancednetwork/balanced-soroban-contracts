use soroban_sdk::{contractclient, Env, String, Vec};

use crate::errors::ContractError;

#[contractclient(name = "XcallManagerClient")]
pub trait IXcallManager {
    fn verify_protocols(e: Env, protocols: Vec<String>) -> Result<bool, ContractError>;

    fn get_protocols(e: Env) -> Result<(Vec<String>, Vec<String>), ContractError>;
}
