#![cfg(test)]
extern crate std;

use crate::contract::{AssetManager, AssetManagerClient};

use crate::config::ConfigData;

use soroban_sdk::Vec;
use soroban_sdk::{testutils::Address as _, token, Address, Env, String};

mod xcall {
    soroban_sdk::contractimport!(file = "../../wasm/xcall.wasm");
}

mod connection {
    soroban_sdk::contractimport!(file = "../../wasm/centralized_connection.wasm");
}

mod xcall_manager {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/xcall_manager.wasm"
    );
}

use xcall_manager::ConfigData as XcallManagerConfigData;

pub struct TestContext {
    pub env: Env,
    pub registry: Address,
    pub admin: Address,
    pub depositor: Address,
    pub withdrawer: Address,
    pub upgrade_authority: Address,
    pub xcall: Address,
    pub xcall_manager: Address,
    pub icon_asset_manager: String,
    pub icon_governance: String,
    pub token: Address,
    pub centralized_connection: Address,
    pub nid: String,
    pub native_token: Address,
    pub xcall_client: xcall::Client<'static>,
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();
        let token_admin = Address::generate(&env);
        let token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let asset_manager = env.register_contract(None, AssetManager);
        let centralized_connection = env.register_contract_wasm(None, connection::WASM);
        let xcall_manager = env.register_contract_wasm(None, xcall_manager::WASM);
        let xcall = env.register_contract_wasm(None, xcall::WASM);

        Self {
            registry: asset_manager,
            admin: Address::generate(&env),
            depositor: Address::generate(&env),
            withdrawer: Address::generate(&env),
            upgrade_authority: Address::generate(&env),
            xcall: xcall.clone(),
            xcall_manager: xcall_manager,
            icon_asset_manager: String::from_str(&env, "icon01/hxjnfh4u"),
            icon_governance: String::from_str(&env, "icon01/kjdnoi"),
            token: token.address(),
            centralized_connection: centralized_connection,
            nid: String::from_str(&env, "stellar"),
            native_token: env
                .register_stellar_asset_contract_v2(token_admin.clone())
                .address(),
            xcall_client: xcall::Client::new(&env, &xcall),
            env,
        }
    }

    pub fn init_context(&self, client: &AssetManagerClient<'static>) {
        self.env.mock_all_auths();
        self.init_xcall_manager_context();
        self.init_xcall_state();
        let config = ConfigData {
            xcall: self.xcall.clone(),
            xcall_manager: self.xcall_manager.clone(),
            native_address: self.native_token.clone(),
            icon_asset_manager: self.icon_asset_manager.clone(),
            xcall_network_address: self.xcall_client.get_network_address(),
            upgrade_authority: self.upgrade_authority.clone(),
        };
        client.initialize(&self.registry, &self.admin, &config);
    }

    pub fn init_xcall_manager_context(&self) {
        let client = self::xcall_manager::Client::new(&self.env, &self.xcall_manager);
        let config = XcallManagerConfigData {
            xcall: self.xcall.clone(),
            icon_governance: self.icon_governance.clone(),
            upgrade_authority: self.upgrade_authority.clone(),
        };
        let sources = Vec::from_array(&self.env, [self.centralized_connection.to_string()]);
        let destinations =
            Vec::from_array(&self.env, [String::from_str(&self.env, "icon/address")]);
        client.initialize(
            &self.xcall_manager,
            &self.admin,
            &config,
            &sources,
            &destinations,
        );
    }

    pub fn init_xcall_state(&self) {
        self.xcall_client.initialize(&xcall::InitializeMsg {
            sender: self.admin.clone(),
            network_id: self.nid.clone(),
            native_token: self.native_token.clone(),
        });

        self.init_connection_state();
        self.xcall_client.set_protocol_fee(&100);
        self.xcall_client
            .set_default_connection(&self.nid, &self.centralized_connection);
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

    pub fn mint_native_token(&self, address: &Address, amount: u128) {
        let native_token_client = token::StellarAssetClient::new(&self.env, &self.native_token);
        native_token_client.mint(&address, &(*&amount as i128));
    }

    pub fn get_native_token_balance(&self, address: &Address) -> u128 {
        let native_token_client = token::TokenClient::new(&self.env, &self.native_token);
        let balance = native_token_client.balance(address);

        *&balance as u128
    }
}
