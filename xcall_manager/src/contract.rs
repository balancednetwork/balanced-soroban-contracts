use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec, Address, BytesN, String, Map};

use crate::{
    admin:: {self, has_administrator, read_administrator, write_administrator}, 
    states::{has_state, read_address_state, read_i128_state, read_string_state, read_u64_state, write_address_state, write_i128_state, write_string_state, write_u64_state,
    write_vec_string_state, read_vec_string_state },
    storage_types::{DataKey, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD }
};

pub mod xcall {
    soroban_sdk::contractimport!(
        file = "../xcall/target/wasm32-unknown-unknown/release/xcall.wasm"
    );
}

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
            
            env.storage().instance().set(&DataKey::Xcall, &xcall);
            env.storage().instance().set(&DataKey::IconGovernance, &icon_governance);
            env.storage().instance().set(&DataKey::XcallNetworkAddress, &xcall_network_address);
            env.storage().instance().set(&DataKey::ProposedProtocolToRemove, &proposed_protocol_to_remove);
            env.storage().instance().set(&DataKey::Sources, &sources);
            env.storage().instance().set(&DataKey::Destinations, &destinations);
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

    pub fn set_protocols(e: Env, sources: Vec<String>, destinations: Vec<String>) {
        let admin = read_administrator(&e);
        admin.require_auth();

        write_vec_string_state(&e, DataKey::Sources, &sources);
        write_vec_string_state(&e, DataKey::Destinations, &destinations);
    }

    pub fn verify_protocols(
        e: Env,
        protocols: Vec<String>
    ) -> bool {
        let sources = read_vec_string_state(&e, DataKey::Sources);
        return Self::verify_protocols_unordered(e, protocols, sources);
    }

    pub fn get_protocols(e: Env) -> Map<String, Vec<String>> {
        let sources = read_vec_string_state(&e, DataKey::Sources);
        let destinations = read_vec_string_state(&e, DataKey::Destinations);
         let mut protocols = Map::new(&e);
         protocols.set(String::from_str(&e, "sources"), sources);
         protocols.set(String::from_str(&e, "destinations"), destinations);
         protocols
    }

    pub fn verify_protocols_unordered(_e: Env, array1: Vec<String>, array2: Vec<String>) -> bool {
        // Check if the arrays have the same length
        if array1.len() != array2.len() {
            return false;
        }
        for p in array1.iter() {
            let mut j = 0;
            for s in array2.iter() {
                j = j+1;
                if p.eq(&s) {
                    break;
                } else {
                    if j == array2.len()  {
                         return false; 
                    }
                    continue;
                }
                
            }
        }
        return true;
    }

    pub fn handle_call_message(
        e: Env,
        from: String,
        data: BytesN<32>,
        protocols: Vec<String>
    ) {
        if !from.eq(&read_string_state(&e, DataKey::IconGovernance)) {
          panic!("Only ICON Balanced governance is allowed")
        }

        // string memory method = data.getMethod();
        // if (!verifyProtocolsUnordered(protocols, sources)) {
        //     require(
        //         method.compareTo(Messages.CONFIGURE_PROTOCOLS_NAME),
        //         "Protocol Mismatch"
        //     );
        //     verifyProtocolRecovery(protocols);
        // }

        // if (method.compareTo(Messages.EXECUTE_NAME)) {
        //     Messages.Execute memory message = data.decodeExecute();
        //     (bool _success, ) = message.contractAddress.call(message.data);
        //     require(_success, "Failed to excute message");
        // } else if (method.compareTo(Messages.CONFIGURE_PROTOCOLS_NAME)) {
        //     Messages.ConfigureProtocols memory message = data
        //         .decodeConfigureProtocols();
        //     sources = message.sources;
        //     destinations = message.destinations;
        // } else {
        //     revert("Unknown message type");
        // }
    }

    pub fn verify_protocol_recovery(e: Env, protocols: Vec<String>) {
        let modifiedSources = Self::get_modified_protocols(e.clone());
        if !Self::verify_protocols_unordered(e.clone(), modifiedSources, protocols) {
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