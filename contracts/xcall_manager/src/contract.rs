use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec, panic_with_error};
mod xcall {
    soroban_sdk::contractimport!(file = "../../wasm/xcall.wasm" );
}
use soroban_rlp::messages::configure_protocols::ConfigureProtocols;
use crate::{
    config::{get_config, set_config, ConfigData}, 
    states::{has_registry, has_proposed_removed, read_administrator, write_administrator, write_registry,
        read_destinations, write_destinations, read_sources, write_sources, read_proposed_removed, write_proposed_removed, extend_ttl}, 
};

use crate::errors::ContractError;

const CONFIGURE_PROTOCOLS_NAME: &str = "ConfigureProtocols";

#[contract]
pub struct XcallManager;

#[contractimpl]
impl XcallManager {
    
    pub fn initialize(env:Env, registry:Address, admin: Address, config: ConfigData, sources: Vec<String>, destinations: Vec<String>) {
        if has_registry(env.clone()) {
            panic_with_error!(env, ContractError::ContractAlreadyInitialized)
        }
        write_registry(&env, &registry);
        write_administrator(&env, &admin);
        Self::configure(env, config, sources, destinations );
    }

    pub fn configure(env:Env, config: ConfigData, sources: Vec<String>, destinations: Vec<String>){
        let admin = read_administrator(&env.clone());
        admin.require_auth();

        set_config(&env, config);
        write_sources(&env, &sources);
        write_destinations(&env, &destinations);
    }

    pub fn get_config(env: Env) -> ConfigData{
        get_config(&env)
    }

    pub fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        write_administrator(&e, &new_admin);
    }

    pub fn get_admin(e: Env) -> Address {
        read_administrator(&e)
    }

    pub fn propose_removal(e: Env, protocol: String) {
        let admin = read_administrator(&e);
        admin.require_auth();
        
        write_proposed_removed(&e, &protocol);     
    }

    pub fn get_proposed_removal(e: Env) -> String {
        read_proposed_removed(&e)
    }

    pub fn verify_protocols(
        e: Env,
        protocols: Vec<String>
    )  -> Result<bool, ContractError> {
        let sources: Vec<String> = read_sources(&e);
        return Self::verify_protocols_unordered(e, protocols, sources);
    }

    pub fn get_protocols(e: Env) -> Result<(Vec<String>, Vec<String>), ContractError> {
        let sources = read_sources(&e);
        let destinations = read_destinations(&e);
        Ok((sources, destinations))
    }

    pub fn verify_protocols_unordered(_e: Env, array1: Vec<String>, array2: Vec<String>) -> Result<bool, ContractError> {
        // Check if the arrays have the same length
        if array1.len() != array2.len() {
            return Ok(false);
        }
        for p in array1.iter() {
            let mut j = 0;
            for s in array2.iter() {
                j = j+1;
                if p.eq(&s) {
                    break;
                } else {
                    if j == array2.len()  {
                         return Ok(false); 
                    }
                    continue;
                }
                
            }
        }
        return Ok(true);
    }

    pub fn handle_call_message(
        e: Env,
        from: String,
        data: Bytes,
        protocols: Vec<String>
    ) {
        let xcall = get_config(&e.clone()).xcall;
        xcall.require_auth();

        let icon_governance = get_config(&e.clone()).icon_governance;
        if from != icon_governance {
            panic_with_error!(e, ContractError::OnlyICONGovernance)
        }

        
        if !Self::verify_protocols(e.clone(), protocols.clone()).unwrap() {
            panic_with_error!(e, ContractError::ProtocolMismatch)
        };

        let method = ConfigureProtocols::get_method(&e.clone(), data.clone());

        let sources = read_sources(&e);
        if !Self::verify_protocols_unordered(e.clone(), protocols.clone(), sources).unwrap() {
                if method != String::from_str(&e.clone(), CONFIGURE_PROTOCOLS_NAME)  {
                    panic_with_error!(e, ContractError::ProtocolMismatch)
                }
            Self::verify_protocol_recovery(e.clone(), protocols);
        }

        if method == String::from_str(&e, CONFIGURE_PROTOCOLS_NAME) {
            let message = ConfigureProtocols::decode(&e, data);
            let sources = message.sources;
            let destinations = message.destinations;
            write_sources(&e, &sources);
            write_destinations(&e, &destinations);
        } else {
            panic_with_error!(e, ContractError::UnknownMessageType)
        }
    }

    pub fn verify_protocol_recovery(e: Env, protocols: Vec<String>) {
        let modified_sources = Self::get_modified_protocols(e.clone());
        let verify_unordered = Self::verify_protocols_unordered(e.clone(), modified_sources, protocols).unwrap();
        if !verify_unordered {
            panic_with_error!(e, ContractError::ProtocolMismatch)
        }
    }

    pub fn get_modified_protocols(e: Env) -> Vec<String>{
        if !has_proposed_removed(e.clone()) {
            panic_with_error!(e, ContractError::NoProposalForRemovalExists)
        }

        let sources = read_sources(&e);
        let protocol_to_remove = read_proposed_removed(&e);
        let mut new_array = Vec::new(&e);
        for s in sources.iter() {
            if !s.eq(&protocol_to_remove) {
                new_array.push_back(s);
            }
        }
        
        return new_array;
    } 

    pub fn extend_ttl(e: Env){
        extend_ttl(&e);
    }
}