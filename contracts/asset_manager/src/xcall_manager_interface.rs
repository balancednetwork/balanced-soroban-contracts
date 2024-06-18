use soroban_sdk::{contractclient, Env, String, Vec};

use crate::errors::ContractError;

#[contractclient(name = "XcallManagerClient")]
pub trait XcallManagerInterface {

    fn verify_protocols(
        e: Env,
        protocols: Vec<String>
    )  -> bool;

    fn get_protocols(e: Env) -> Result<(Vec<String>, Vec<String>), ContractError>;
    
}