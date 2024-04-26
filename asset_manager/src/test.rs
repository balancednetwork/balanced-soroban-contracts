#![cfg(test)]

extern crate std;

use crate::contract::{
    AssetManager, AssetManagerClient, xcall_manager
};

use crate::storage_types::{DataKey, POINTS};

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    token, vec, Address, Bytes, Env, IntoVal, String, Symbol,
};

pub struct TestContext {
    env: Env, 
    registry:Address, 
    admin: Address, 
    xcall: Address, 
    xcall_manager: Address, 
    native_address: Address, 
    icon_asset_manager: String, 
    xcall_network_address: String
}

pub struct TokenRateLimit {
    token: Address,
    period: i128,
    percentage: i128
}

impl TestContext {
    pub fn default() -> Self {
        let env = Env::default();
        let token_admin = Address::generate(&env);
        Self {
            registry: env.register_contract(None, AssetManager),
            admin: Address::generate(&env),
            xcall: Address::generate(&env),
            xcall_manager: env.register_contract_wasm(None, xcall_manager::WASM), 
            native_address: env.register_stellar_asset_contract(token_admin.clone()),
            icon_asset_manager: String::from_str(&env, "icon_asset_manager"),
            xcall_network_address: String::from_str(&env, "xcall_network_address"),
            env
        }
    }

    pub fn init_context(&self, client: &AssetManagerClient<'static>) {
        self.env.mock_all_auths();
        client.initialize(&self.registry, &self.admin, &self.xcall, &self.xcall_manager, &self.native_address, &self.icon_asset_manager );
    }
}

#[test]
fn test_initialize() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);

    ctx.init_context(&client);

    let registry_exists = client.has_registry();
    assert_eq!(registry_exists, true)
}