#![cfg(test)]
extern crate std;

use crate::contract::{XcallManager, XcallManagerClient};

use crate::config::ConfigData;

use soroban_sdk::Vec;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

mod xcall {
    soroban_sdk::contractimport!(file = "../../wasm/xcall.wasm");
}

mod connection {
    soroban_sdk::contractimport!(file = "../../wasm/centralized_connection.wasm");
}

pub struct TestContext {
    pub env: Env,
    pub registry: Address,
    pub admin: Address,
    pub depositor: Address,
    pub upgrade_authority: Address,
    pub xcall: Address,
    pub icon_governance: String,
    pub xcall_network_address: String,
    pub token: Address,
    pub centralized_connection: Address,
    pub nid: String,
    pub native_token: Address,
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();
        let token_admin = Address::generate(&env);
        let token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let xcall_manager = env.register_contract(None, XcallManager);
        let centralized_connection = env.register_contract_wasm(None, connection::WASM);
        let xcall = env.register_contract_wasm(None, xcall::WASM);
        std::println!("xcall manager{:?}", xcall_manager);
        std::println!("xcall {:?}", xcall);
        std::println!("centralized {:?}", centralized_connection);

        Self {
            registry: xcall_manager,
            admin: Address::generate(&env),
            depositor: Address::generate(&env),
            upgrade_authority: Address::generate(&env),
            xcall,
            icon_governance: String::from_str(&env, "icon01/kjdnoi"),
            xcall_network_address: String::from_str(&env, "stellar/address"),
            token: token.address(),
            centralized_connection,
            nid: String::from_str(&env, "stellar"),
            native_token: env
                .register_stellar_asset_contract_v2(token_admin.clone())
                .address(),
            env,
        }
    }

    pub fn init_context(&self, client: &XcallManagerClient<'static>) {
        self.env.mock_all_auths();
        let config = ConfigData {
            xcall: self.xcall.clone(),
            icon_governance: self.icon_governance.clone(),
            upgrade_authority: self.upgrade_authority.clone(),
        };
        let sources = Vec::from_array(&self.env, [self.centralized_connection.to_string()]);
        let destinations =
            Vec::from_array(&self.env, [String::from_str(&self.env, "icon/address")]);
        client.initialize(
            &self.registry,
            &self.admin,
            &config,
            &sources,
            &destinations,
        );
        self.init_xcall_state();
    }

    pub fn init_xcall_state(&self) {
        let xcall_client = xcall::Client::new(&self.env, &self.xcall);

        xcall_client.initialize(&xcall::InitializeMsg {
            sender: self.admin.clone(),
            network_id: self.nid.clone(),
            native_token: self.native_token.clone(),
        });

        self.init_connection_state();
        xcall_client.set_protocol_fee(&100);
        xcall_client.set_default_connection(&self.nid, &self.centralized_connection);
    }

    pub fn init_connection_state(&self) {
        let connection_client = connection::Client::new(&self.env, &self.centralized_connection);

        let initialize_msg = connection::InitializeMsg {
            native_token: self.native_token.clone(),
            relayer: self.admin.clone(),
            xcall_address: self.xcall.clone(),
        };
        connection_client.initialize(&initialize_msg);

        let message_fee = 100;
        let response_fee = 100;
        connection_client.set_fee(&self.nid, &message_fee, &response_fee);
    }
}
