use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, Bytes, BytesN, Env, String, Vec,
};
mod xcall {
    soroban_sdk::contractimport!(file = "../../wasm/xcall.wasm");
}

use crate::{
    config::{self, get_config, set_config, ConfigData},
    states::{
        extend_ttl, has_proposed_removed, has_registry, read_administrator, read_destinations,
        read_proposed_removed, read_sources, write_administrator, write_destinations,
        write_proposed_removed, write_registry, write_sources,
    },
    storage_types::DataKey,
    white_list_actions::WhiteListActions,
};
use soroban_rlp::balanced::messages::configure_protocols::ConfigureProtocols;

use crate::errors::ContractError;

const CONFIGURE_PROTOCOLS_NAME: &str = "ConfigureProtocols";

#[contract]
pub struct XcallManager;

#[contractimpl]
impl XcallManager {
    pub fn initialize(
        env: Env,
        registry: Address,
        admin: Address,
        config: ConfigData,
        sources: Vec<String>,
        destinations: Vec<String>,
    ) {
        if has_registry(env.clone()) {
            panic_with_error!(env, ContractError::ContractAlreadyInitialized)
        }
        write_registry(&env, &registry);
        write_administrator(&env, &admin);
        set_config(&env, config);
        write_sources(&env, &sources);
        write_destinations(&env, &destinations);
    }

    pub fn get_config(env: Env) -> ConfigData {
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

    pub fn white_list_actions(e: Env, action: Bytes) {
        let actions = WhiteListActions::new(DataKey::WhiteListedActions);
        actions.add(&e, action);
    }

    pub fn remove_action(e: Env, action: Bytes) -> Result<bool, ContractError> {
        let actions = WhiteListActions::new(DataKey::WhiteListedActions);
        if !actions.contains(&e, action.clone()) {
            return Err(ContractError::NotWhiteListed);
        }
        actions.remove(&e, action);
        Ok(true)
    }

    pub fn verify_protocols(e: Env, protocols: Vec<String>) -> Result<bool, ContractError> {
        let sources: Vec<String> = read_sources(&e);

        let verified = Self::verify_protocols_unordered(protocols, sources)?;
        return Ok(verified);
    }

    pub fn get_protocols(e: Env) -> Result<(Vec<String>, Vec<String>), ContractError> {
        let sources: Vec<String> = read_sources(&e);
        let destinations = read_destinations(&e);
        Ok((sources, destinations))
    }

    pub fn verify_protocols_unordered(
        array1: Vec<String>,
        array2: Vec<String>,
    ) -> Result<bool, ContractError> {
        // Check if the arrays have the same length
        if array1.len() != array2.len() {
            return Ok(false);
        }
        for p in array2.iter() {
            let mut j = 0;
            for s in array1.iter() {
                j = j + 1;
                if p.eq(&s) {
                    break;
                } else {
                    if j == array1.len() {
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
        protocols: Vec<String>,
    ) -> Result<(), ContractError> {
        let config = get_config(&e.clone());
        let xcall = config.xcall;
        xcall.require_auth();

        if from != config.icon_governance {
            return Err(ContractError::OnlyICONGovernance);
        }

        let actions = WhiteListActions::new(DataKey::WhiteListedActions);
        if !actions.contains(&e, data.clone()) {
            return Err(ContractError::NotWhiteListed);
        }
        actions.remove(&e, data.clone());

        if !Self::verify_protocols(e.clone(), protocols.clone())? {
            return Err(ContractError::ProtocolMismatch);
        };

        let method = ConfigureProtocols::get_method(&e.clone(), data.clone());

        let sources = read_sources(&e);
        if !Self::verify_protocols_unordered(protocols.clone(), sources).unwrap() {
            if method != String::from_str(&e.clone(), CONFIGURE_PROTOCOLS_NAME) {
                return Err(ContractError::UnknownMessageType);
            }
            Self::verify_protocol_recovery(&e, protocols)?;
        }

        if method == String::from_str(&e, CONFIGURE_PROTOCOLS_NAME) {
            let message = ConfigureProtocols::decode(&e, data);
            let sources = message.sources;
            let destinations = message.destinations;
            write_sources(&e, &sources);
            write_destinations(&e, &destinations);
        } else {
            return Err(ContractError::UnknownMessageType);
        }
        Ok(())
    }

    pub fn verify_protocol_recovery(e: &Env, protocols: Vec<String>) -> Result<(), ContractError> {
        let modified_sources = Self::get_modified_protocols(e)?;
        let verify_unordered =
            Self::verify_protocols_unordered(modified_sources, protocols).unwrap();
        if !verify_unordered {
            return Err(ContractError::ProtocolMismatch);
        }
        Ok(())
    }

    pub fn get_modified_protocols(e: &Env) -> Result<Vec<String>, ContractError> {
        if !has_proposed_removed(e.clone()) {
            return Err(ContractError::NoProposalForRemovalExists);
        }

        let sources = read_sources(&e);
        let protocol_to_remove = read_proposed_removed(&e);
        let mut new_array = Vec::new(&e);
        for s in sources.iter() {
            if !s.eq(&protocol_to_remove) {
                new_array.push_back(s);
            }
        }

        return Ok(new_array);
    }

    pub fn set_upgrade_authority(e: Env, upgrade_authority: Address) {
        let mut config = config::get_config(&e);

        config.upgrade_authority.require_auth();

        config.upgrade_authority = upgrade_authority;
        config::set_config(&e, config);
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let config = get_config(&e);
        config.upgrade_authority.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    pub fn extend_ttl(e: Env) {
        extend_ttl(&e);
    }
}
