use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec};
mod xcall {
    soroban_sdk::contractimport!(file = "xcall.wasm" );
}

use crate::{
    admin:: {read_administrator, write_administrator}, 
    config::{get_config, ConfigData}, 
    states::{has_state, read_string_state, read_vec_string_state, write_string_state, write_vec_string_state }, 
    storage_types::{DataKey, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD },
    messages::{configure_protocols::ConfigureProtocols, execute::Execute }
    
};
use crate::errors::ContractError;

const CONFIGURE_PROTOCOLS_NAME: &str = "ConfigureProtocols";
const EXECUTE_NAME: &str = "Execute";

#[contract]
pub struct XcallManager;

#[contractimpl]
impl XcallManager {
    
    pub fn initialize(env:Env, registry:Address, admin: Address, xcall: Address, 
        icon_governance: String, xcall_network_address: String, proposed_protocol_to_remove: String, sources: Vec<String>, destinations: Vec<String>) {
        if has_state(env.clone(), DataKey::Registry) {
            panic!("Contract already initialized.")
        }
        env.storage().instance().set(&DataKey::Registry, &registry);
        env.storage().instance().set(&DataKey::Admin, &admin);
        Self::configure(env, xcall, icon_governance, xcall_network_address, proposed_protocol_to_remove, sources, destinations );
    }

    pub fn configure(env:Env, xcall: Address, 
         icon_governance: String, xcall_network_address: String, proposed_protocol_to_remove: String, sources: Vec<String>, destinations: Vec<String>){
        let admin = read_administrator(&env.clone());
        admin.require_auth();

        let config: ConfigData = ConfigData{ xcall: xcall, icon_governance: icon_governance, xcall_network_address: xcall_network_address };
        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::Sources, &sources);
        env.storage().instance().set(&DataKey::Destinations, &destinations);
        env.storage().instance().set(&DataKey::ProposedProtocolToRemove, &proposed_protocol_to_remove);
    }



    pub fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_administrator(&e, &new_admin);
    }

    pub fn propose_removal(e: Env, protocol: String) {
        let admin = read_administrator(&e);
        admin.require_auth();

        write_string_state(&e, DataKey::ProposedProtocolToRemove, &protocol);     
    }

    pub fn verify_protocols(
        e: Env,
        protocols: Vec<String>
    )  -> Result<bool, ContractError> {
        let sources = read_vec_string_state(&e, DataKey::Sources);
        return Self::verify_protocols_unordered(e, protocols, sources);
    }

    pub fn get_protocols(e: Env) -> Result<(Vec<String>, Vec<String>), ContractError> {
        let sources = read_vec_string_state(&e, DataKey::Sources);
        let destinations = read_vec_string_state(&e, DataKey::Destinations);
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
            panic!("Only ICON Balanced governance is allowed")
        }

        
        if !Self::verify_protocols(e.clone(), protocols.clone()).unwrap() {
            panic!("Protocol Mismatch");
        };

        let method = ConfigureProtocols::get_method(&e.clone(), data.clone()).unwrap();

        let sources = read_vec_string_state(&e, DataKey::Sources);
        if !Self::verify_protocols_unordered(e.clone(), protocols.clone(), sources).unwrap() {
                if method != String::from_str(&e.clone(), CONFIGURE_PROTOCOLS_NAME)  {
                    panic!("Protocol Mismatch");
                }
            Self::verify_protocol_recovery(e.clone(), protocols);
        }

        if method == String::from_str(&e.clone(),  EXECUTE_NAME) {
            let message = Execute::decode(&e.clone(), data).unwrap();
            // (bool _success, ) = message.contractAddress.call(message.data);
            // require(_success, "Failed to excute message");
            //e.invoke_contract(&message.contract_address, &Symbol::new(&e.clone(), "test"), data);
        } else if method == String::from_str(&e, CONFIGURE_PROTOCOLS_NAME) {
            let message = ConfigureProtocols::decode(&e, data).unwrap();
            let sources = message.sources;
            let destinations = message.destinations;
            write_vec_string_state(&e, DataKey::Sources, &sources);
            write_vec_string_state(&e, DataKey::Destinations, &destinations);
        } else {
            panic!("Unknown message type");
        }
    }

    pub fn verify_protocol_recovery(e: Env, protocols: Vec<String>) {
        let modified_sources = Self::get_modified_protocols(e.clone());
        let verify_unordered = Self::verify_protocols_unordered(e.clone(), modified_sources, protocols).unwrap();
        if !verify_unordered {
           panic!("Protocol Mismatch")
        }
    }


    pub fn get_modified_protocols(e: Env) -> Vec<String>{
        if !has_state(e.clone(), DataKey::ProposedProtocolToRemove) {
            panic!( "No proposal for removal exists")
        }

        let sources = read_vec_string_state(&e, DataKey::Sources);
        let protocol_to_remove = read_string_state(&e, DataKey::ProposedProtocolToRemove);
        let mut new_array = Vec::new(&e);
        for s in sources.iter() {
            if !s.eq(&protocol_to_remove) {
                new_array.push_back(s);
            }
        }
        
        return new_array;
    } 


}