#![cfg(test)]
extern crate std;

use crate::{config, contract::AssetManagerClient, storage_types::DataKey};
use soroban_sdk::{
    testutils::{storage::Persistent, Address as _, AuthorizedFunction, AuthorizedInvocation},
    token, Address, Bytes, IntoVal, String, Symbol, Vec,
};

use soroban_rlp::balanced::messages::{deposit_revert::DepositRevert, withdraw_to::WithdrawTo};

use super::setup::*;

#[test]
fn test_initialize() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);

    ctx.init_context(&client);

    let registry_exists = client.has_registry();
    assert_eq!(registry_exists, true)
}

#[test]
fn test_set_admin() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);

    let new_admin: Address = Address::generate(&ctx.env);
    client.set_admin(&new_admin);
    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.registry.clone(),
                    Symbol::new(&ctx.env, "set_admin"),
                    (&new_admin,).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(client.get_admin(), new_admin);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_configure_rate_limit_panic() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);
    let period = &300;
    let percentage = &10001;
    client.configure_rate_limit(&ctx.token, period, percentage);

    let limit = client.get_withdraw_limit(&ctx.token);
    let verified = client.verify_withdraw(&ctx.token, &limit);
    assert_eq!(verified, true);
}

#[test]
fn test_configure_rate_limit() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);
    let period = &300;
    let percentage = &300;
    client.configure_rate_limit(&ctx.token, period, percentage);
    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.registry.clone(),
                    Symbol::new(&ctx.env, "configure_rate_limit"),
                    (&ctx.token, 300u64, 300u32).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    let token_data = client.get_rate_limit(&ctx.token);
    assert_eq!(token_data.3, 0);
    let limit = client.get_withdraw_limit(&ctx.token);
    let verified = client.verify_withdraw(&ctx.token, &limit);
    assert_eq!(verified, true);
}

#[test]
fn test_deposit_without_to_and_data() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);

    client.configure_rate_limit(&ctx.token, &300, &300);
    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    let amount_i128: i128 = 100000i128;
    let amount = &(amount_i128 as u128);
    let mint_amount = &(amount_i128 + amount_i128);

    stellar_asset_client.mint(&ctx.depositor, mint_amount);

    ctx.mint_native_token(&ctx.depositor, 500);
    assert_eq!(ctx.get_native_token_balance(&ctx.depositor), 500);

    token_client.approve(
        &ctx.depositor,
        &ctx.registry,
        &(amount_i128 + amount_i128),
        &1312000,
    );
    client.deposit(
        &ctx.depositor,
        &ctx.token,
        &amount,
        &Option::Some(String::from_str(&ctx.env, "")),
        &Option::Some(Bytes::from_array(&ctx.env, &[0u8; 32])),
    );

    assert_eq!(ctx.get_native_token_balance(&ctx.depositor), 400); // why 300?
}

#[test]
fn test_veryfy_rate_limit() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);
    let period = &300;
    let percentage = &300;
    client.configure_rate_limit(&ctx.token, period, percentage);

    //let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    let amount_i128: i128 = 100000i128;
    let amount = &(amount_i128 as u128);
    let mint_amount: &i128 = &(amount_i128 + amount_i128);

    stellar_asset_client.mint(&ctx.depositor, mint_amount);

    ctx.mint_native_token(&ctx.depositor, 500u128);
    assert_eq!(ctx.get_native_token_balance(&ctx.depositor), 500u128);

    //token_client.approve(&ctx.depositor, &ctx.registry, &(amount_i128+amount_i128), &1312000);
    client.deposit(
        &ctx.depositor,
        &ctx.token,
        &amount,
        &Option::Some(String::from_str(&ctx.env, "")),
        &Option::Some(Bytes::from_array(&ctx.env, &[0u8; 32])),
    );

    let limit = client.get_withdraw_limit(&ctx.token);
    assert_eq!(limit, 3000);
    let verified = client.verify_withdraw(&ctx.token, &(amount - 3000 - 1));
    assert_eq!(verified, true);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_veryfy_rate_limit_panic_exceeds_withdraw_limit() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);
    let period = &300;
    let percentage = &300;
    client.configure_rate_limit(&ctx.token, period, percentage);

    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    let amount_i128: i128 = 100000i128;
    let amount = &(amount_i128 as u128);
    let mint_amount = &(amount_i128 + amount_i128);

    stellar_asset_client.mint(&ctx.depositor, mint_amount);

    ctx.mint_native_token(&ctx.depositor, 500u128);
    assert_eq!(ctx.get_native_token_balance(&ctx.depositor), 500u128);

    token_client.approve(
        &ctx.depositor,
        &ctx.registry,
        &(amount_i128 + amount_i128),
        &1312000,
    );
    client.deposit(
        &ctx.depositor,
        &ctx.token,
        &amount,
        &Option::Some(String::from_str(&ctx.env, "")),
        &Option::Some(Bytes::from_array(&ctx.env, &[0u8; 32])),
    );

    let limit = client.get_withdraw_limit(&ctx.token);
    assert_eq!(limit, 3000);
    let verified = client.verify_withdraw(&ctx.token, &(amount - 3000 + 1));
    assert_eq!(verified, true);
}

#[test]
fn test_deposit_with_to_and_without_data() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);

    client.configure_rate_limit(&ctx.token, &300, &300);
    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    let amount_i128: i128 = 100000i128;
    let amount = &(amount_i128 as u128);
    let mint_amount = &(amount_i128 + amount_i128);

    stellar_asset_client.mint(&ctx.depositor, mint_amount);

    ctx.mint_native_token(&ctx.depositor, 500);
    assert_eq!(ctx.get_native_token_balance(&ctx.depositor), 500);

    token_client.approve(
        &ctx.depositor,
        &ctx.registry,
        &(amount_i128 + amount_i128),
        &1312000,
    );
    client.deposit(
        &ctx.depositor,
        &ctx.token,
        &amount,
        &Option::Some(String::from_str(&ctx.env, "icon01/hxjkdvhui")),
        &Option::Some(Bytes::from_array(&ctx.env, &[0u8; 32])),
    );

    assert_eq!(ctx.get_native_token_balance(&ctx.depositor), 400) // why 300?
}

#[test]
fn test_deposit_with_to_and_data() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);

    client.configure_rate_limit(&ctx.token, &300, &300);
    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    let amount_i128: i128 = 100000i128;
    let amount = &(amount_i128 as u128);
    let mint_amount = &(amount_i128 + amount_i128);

    stellar_asset_client.mint(&ctx.depositor, mint_amount);

    ctx.mint_native_token(&ctx.depositor, 500);
    assert_eq!(ctx.get_native_token_balance(&ctx.depositor), 500);

    token_client.approve(
        &ctx.depositor,
        &ctx.registry,
        &(amount_i128 + amount_i128),
        &1312000,
    );

    let data: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        0x1F, 0x20,
    ];
    client.deposit(
        &ctx.depositor,
        &ctx.token,
        &amount,
        &Option::Some(String::from_str(&ctx.env, "icon01/hxjkdvhui")),
        &Option::Some(Bytes::from_array(&ctx.env, &data)),
    );
    assert_eq!(ctx.get_native_token_balance(&ctx.depositor), 400); // why 300?
}

#[test]
fn test_handle_call_message_for_withdraw_to() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);
    //client.configure_rate_limit(&ctx.token, &300, &300);

    let bnusd_amount = 100000u128;
    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    stellar_asset_client.mint(&ctx.registry, &((bnusd_amount * 2) as i128));

    let data = WithdrawTo::new(
        ctx.token.to_string(),
        ctx.withdrawer.to_string(),
        bnusd_amount,
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "WithdrawTo"));
    let decoded = WithdrawTo::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, ctx.withdrawer.to_string());

    assert_eq!(token_client.balance(&ctx.withdrawer), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.icon_asset_manager, &data, &sources);

    assert_eq!(token_client.balance(&ctx.withdrawer), bnusd_amount as i128)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #13)")]
fn test_handle_call_message_for_withdraw_to_invalid_address() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);
    client.configure_rate_limit(&ctx.token, &300, &300);

    let bnusd_amount = 100000u128;
    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    stellar_asset_client.mint(&ctx.registry, &((bnusd_amount * 2) as i128));

    let data = WithdrawTo::new(
        ctx.token.to_string(),
        String::from_str(&ctx.env, "InvalidAddress"),
        bnusd_amount,
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "WithdrawTo"));
    let decoded = WithdrawTo::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, String::from_str(&ctx.env, "InvalidAddress"));

    assert_eq!(token_client.balance(&ctx.withdrawer), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.icon_asset_manager, &data, &sources);

    assert_eq!(token_client.balance(&ctx.withdrawer), bnusd_amount as i128)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_handle_call_message_for_withdraw_to_panic_with_protocal_mismatch() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);
    client.configure_rate_limit(&ctx.token, &300, &300);

    let bnusd_amount = 100000u128;
    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    stellar_asset_client.mint(&ctx.registry, &((bnusd_amount * 2) as i128));

    let data = WithdrawTo::new(
        ctx.token.to_string(),
        ctx.withdrawer.to_string(),
        bnusd_amount,
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "WithdrawTo"));
    let decoded = WithdrawTo::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, ctx.withdrawer.to_string());

    assert_eq!(token_client.balance(&ctx.withdrawer), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.xcall.to_string()]);
    client.handle_call_message(&ctx.icon_asset_manager, &data, &sources);

    assert_eq!(token_client.balance(&ctx.withdrawer), bnusd_amount as i128)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_handle_call_message_for_withdraw_to_panic_with_not_icon_asset_manager() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);
    client.configure_rate_limit(&ctx.token, &300, &300);

    let bnusd_amount = 100000u128;
    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    stellar_asset_client.mint(&ctx.registry, &((bnusd_amount * 2) as i128));

    let data = WithdrawTo::new(
        ctx.token.to_string(),
        ctx.withdrawer.to_string(),
        bnusd_amount,
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "WithdrawTo"));
    let decoded = WithdrawTo::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, ctx.withdrawer.to_string());

    assert_eq!(token_client.balance(&ctx.withdrawer), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.centralized_connection.to_string(), &data, &sources);

    assert_eq!(token_client.balance(&ctx.withdrawer), bnusd_amount as i128)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_handle_call_message_for_withdraw_to_panic_with_unknown_message_type() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);
    client.configure_rate_limit(&ctx.token, &300, &300);

    let bnusd_amount = 100000u128;
    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    stellar_asset_client.mint(&ctx.registry, &((bnusd_amount * 2) as i128));

    let data = WithdrawTo::new(
        ctx.token.to_string(),
        ctx.withdrawer.to_string(),
        bnusd_amount,
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "WithdrawToUnknown"));
    let decoded = WithdrawTo::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, ctx.withdrawer.to_string());

    assert_eq!(token_client.balance(&ctx.withdrawer), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.icon_asset_manager, &data, &sources);

    assert_eq!(token_client.balance(&ctx.withdrawer), bnusd_amount as i128)
}

#[test]
fn test_handle_call_message_for_deposit_rollback() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);
    client.configure_rate_limit(&ctx.token, &300, &300);

    let bnusd_amount = 100000u128;
    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    stellar_asset_client.mint(&ctx.registry, &((bnusd_amount * 2) as i128));

    let data = DepositRevert::new(ctx.token, ctx.withdrawer.clone(), bnusd_amount)
        .encode(&ctx.env, String::from_str(&ctx.env, "DepositRevert"));
    let decoded = DepositRevert::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, ctx.withdrawer);

    assert_eq!(token_client.balance(&ctx.withdrawer), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.xcall_client.get_network_address(), &data, &sources);

    assert_eq!(token_client.balance(&ctx.withdrawer), bnusd_amount as i128)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_handle_call_message_for_deposit_rollback_panic_with_only_call_service() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);
    client.configure_rate_limit(&ctx.token, &300, &300);

    let bnusd_amount = 100000u128;
    let token_client = token::Client::new(&ctx.env, &ctx.token);
    let stellar_asset_client: token::StellarAssetClient =
        token::StellarAssetClient::new(&ctx.env, &ctx.token);
    stellar_asset_client.mint(&ctx.registry, &((bnusd_amount * 2) as i128));

    let data = DepositRevert::new(ctx.token, ctx.withdrawer.clone(), bnusd_amount)
        .encode(&ctx.env, String::from_str(&ctx.env, "DepositRevert"));
    let decoded = DepositRevert::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, ctx.withdrawer);

    assert_eq!(token_client.balance(&ctx.withdrawer), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    let wrong_network_address: String = String::from_str(
        &ctx.env,
        &std::format!(
            "{}/{}",
            "soroban",
            "CBEPDNVYXQGWB5YUBXKJWYJA7OXTZW5LFLNO5JRRGE6Z6C5OSUZPCCEL"
        ),
    );

    std::println!(
        "{}",
        std::string::ToString::to_string(&wrong_network_address)
    );
    client.handle_call_message(&wrong_network_address, &data, &sources);

    assert_eq!(token_client.balance(&ctx.withdrawer), bnusd_amount as i128)
}

#[test]
fn test_extend_ttl() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);

    client.configure_rate_limit(&ctx.token, &300, &300);
    let token = ctx.token;

    client.extend_ttl();

    ctx.env.as_contract(&client.address, || {
        let key = DataKey::TokenData(token.clone());
        let before_ttl = ctx.env.storage().persistent().get_ttl(&key);
        std::println!("before ttl is: {:?}", before_ttl);
    });
}

#[test]
fn test_set_upgrade_authority() {
    let ctx = TestContext::default();
    let client = AssetManagerClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);

    let new_upgrade_authority = Address::generate(&ctx.env);
    client.set_upgrade_authority(&new_upgrade_authority);

    assert_eq!(
        ctx.env.auths(),
        std::vec![(
            ctx.upgrade_authority.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    ctx.registry.clone(),
                    Symbol::new(&ctx.env, "set_upgrade_authority"),
                    (&new_upgrade_authority,).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    ctx.env.as_contract(&client.address, || {
        let config = config::get_config(&ctx.env);
        assert_eq!(config.upgrade_authority, new_upgrade_authority)
    });
}
