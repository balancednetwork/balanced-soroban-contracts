#![cfg(test)]
extern crate std;

use crate::{config, contract::BalancedDollarClient};

use super::setup::*;
use soroban_rlp::balanced::messages::{
    cross_transfer::CrossTransfer, cross_transfer_revert::CrossTransferRevert,
};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Bytes, IntoVal, String, Symbol, Vec,
};

#[test]
fn test_initialize() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);

    ctx.init_context(&client);

    let initialized = client.is_initialized();
    assert_eq!(initialized, true)
}

#[test]
fn test_set_admin() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
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
                    symbol_short!("set_admin"),
                    (&new_admin,).into_val(&ctx.env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(client.get_admin(), new_admin);
}

#[test]
fn test_cross_transfer_with_to_and_data() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
    ctx.init_context(&client);

    let amount_i128: i128 = 100000i128;
    let amount = &(amount_i128 as u128);

    let bnusd_amount = 1000000u128;

    let items: [u8; 32] = [0; 32];
    let to = String::from_str(
        &ctx.env,
        "stellar/CA36FQITV33RO5SJFPTNLRQBD6ZNAEJG7F7J5KWCV4OP7SQHDMIZCT33",
    );
    let from_address = &Address::from_string(&String::from_str(
        &ctx.env,
        "CA36FQITV33RO5SJFPTNLRQBD6ZNAEJG7F7J5KWCV4OP7SQHDMIZCT33",
    ));

    std::println!("to address is: {:?}", to);
    let data = CrossTransfer::new(
        ctx.depositor.to_string(),
        to.clone(),
        bnusd_amount,
        Bytes::from_array(&ctx.env, &items),
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "xCrossTransfer"));

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.icon_bn_usd, &data, &sources);

    ctx.mint_native_token(&from_address, 500u128);
    assert_eq!(ctx.get_native_token_balance(&from_address), 500u128);

    client.approve(
        &from_address,
        &ctx.registry,
        &(amount_i128 + amount_i128),
        &1312000,
    );
    let data: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        0x1F, 0x20,
    ];
    client.cross_transfer(
        &from_address,
        &amount,
        &String::from_str(&ctx.env, "icon01/hxjkdvhui"),
        &Option::Some(Bytes::from_array(&ctx.env, &data)),
    );
    assert_eq!(ctx.get_native_token_balance(&from_address), 400u128) // why 300?
}

#[test]
fn test_handle_call_message_for_cross_transfer() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);

    let bnusd_amount = 100000u128;

    let items: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        0x1F, 0x20,
    ];
    let withdrawer = String::from_str(
        &ctx.env,
        "stellar/CA36FQITV33RO5SJFPTNLRQBD6ZNAEJG7F7J5KWCV4OP7SQHDMIZCT33",
    );
    let data = CrossTransfer::new(
        ctx.depositor.to_string(),
        withdrawer.clone(),
        bnusd_amount,
        Bytes::from_array(&ctx.env, &items),
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "xCrossTransfer"));
    let decoded = CrossTransfer::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, withdrawer);

    let withdrawer_address = &Address::from_string(&String::from_str(
        &ctx.env,
        "CA36FQITV33RO5SJFPTNLRQBD6ZNAEJG7F7J5KWCV4OP7SQHDMIZCT33",
    ));
    assert_eq!(client.balance(withdrawer_address), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.icon_bn_usd, &data, &sources);
    assert_eq!(client.balance(withdrawer_address), bnusd_amount as i128)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_handle_call_message_for_cross_transfer_invalid_addres_fail() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);

    let bnusd_amount = 100000u128;

    let items: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        0x1F, 0x20,
    ];
    let withdrawer = String::from_str(&ctx.env, "stellar/InvalidAddress");
    let data = CrossTransfer::new(
        ctx.depositor.to_string(),
        withdrawer.clone(),
        bnusd_amount,
        Bytes::from_array(&ctx.env, &items),
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "xCrossTransfer"));
    let decoded = CrossTransfer::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, withdrawer);

    // let withdrawer_address = &Address::from_string(&String::from_str(&ctx.env, "InvalidAddress"));
    // assert_eq!(client.balance(withdrawer_address), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.icon_bn_usd, &data, &sources);
    // assert_eq!(client.balance(withdrawer_address), bnusd_amount as i128)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_handle_call_message_for_cross_transfer_panic_for_protocol_mismatch() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);

    let bnusd_amount = 100000u128;

    let items: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        0x1F, 0x20,
    ];
    let withdrawer = String::from_str(
        &ctx.env,
        "stellar/CA36FQITV33RO5SJFPTNLRQBD6ZNAEJG7F7J5KWCV4OP7SQHDMIZCT33",
    );
    let data = CrossTransfer::new(
        ctx.depositor.to_string(),
        withdrawer.clone(),
        bnusd_amount,
        Bytes::from_array(&ctx.env, &items),
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "xCrossTransfer"));
    let decoded = CrossTransfer::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, withdrawer);

    let withdrawer_address = &Address::from_string(&String::from_str(
        &ctx.env,
        "CA36FQITV33RO5SJFPTNLRQBD6ZNAEJG7F7J5KWCV4OP7SQHDMIZCT33",
    ));
    assert_eq!(client.balance(withdrawer_address), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.xcall.to_string()]);
    client.handle_call_message(&ctx.icon_bn_usd, &data, &sources);

    assert_eq!(client.balance(withdrawer_address), bnusd_amount as i128)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_handle_call_message_for_cross_transfer_panic_for_icon_bnusd() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);

    let bnusd_amount = 100000u128;

    let items: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        0x1F, 0x20,
    ];
    let withdrawer = String::from_str(
        &ctx.env,
        "stellar/CA36FQITV33RO5SJFPTNLRQBD6ZNAEJG7F7J5KWCV4OP7SQHDMIZCT33",
    );
    let data = CrossTransfer::new(
        ctx.depositor.to_string(),
        withdrawer.clone(),
        bnusd_amount,
        Bytes::from_array(&ctx.env, &items),
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "xCrossTransfer"));
    let decoded = CrossTransfer::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, withdrawer);

    let withdrawer_address = &Address::from_string(&String::from_str(
        &ctx.env,
        "CA36FQITV33RO5SJFPTNLRQBD6ZNAEJG7F7J5KWCV4OP7SQHDMIZCT33",
    ));
    assert_eq!(client.balance(withdrawer_address), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.icon_governance, &data, &sources);

    assert_eq!(client.balance(withdrawer_address), bnusd_amount as i128)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_handle_call_message_for_cross_transfer_panic_for_wront_message_type() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);

    let bnusd_amount = 100000u128;

    let items: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        0x1F, 0x20,
    ];
    let withdrawer = String::from_str(
        &ctx.env,
        "stellar/CA36FQITV33RO5SJFPTNLRQBD6ZNAEJG7F7J5KWCV4OP7SQHDMIZCT33",
    );
    let data = CrossTransfer::new(
        ctx.depositor.to_string(),
        withdrawer.clone(),
        bnusd_amount,
        Bytes::from_array(&ctx.env, &items),
    )
    .encode(&ctx.env, String::from_str(&ctx.env, "xCrossTransferPanic"));

    let decoded: CrossTransfer = CrossTransfer::decode(&ctx.env, data.clone());
    let withdrawer_address = &Address::from_string(&String::from_str(
        &ctx.env,
        "CA36FQITV33RO5SJFPTNLRQBD6ZNAEJG7F7J5KWCV4OP7SQHDMIZCT33",
    ));
    assert_eq!(decoded.to, withdrawer);

    assert_eq!(client.balance(withdrawer_address), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.icon_bn_usd, &data, &sources);

    assert_eq!(client.balance(withdrawer_address), bnusd_amount as i128)
}

#[test]
fn test_handle_call_message_for_cross_transfer_revert() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);

    let bnusd_amount = 100000u128;

    let data = CrossTransferRevert::new(ctx.withdrawer.clone(), bnusd_amount)
        .encode(&ctx.env, String::from_str(&ctx.env, "xCrossTransferRevert"));
    let decoded = CrossTransferRevert::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, ctx.withdrawer);

    assert_eq!(client.balance(&ctx.withdrawer), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.xcall_client.get_network_address(), &data, &sources);

    assert_eq!(client.balance(&ctx.withdrawer), bnusd_amount as i128)
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn test_handle_call_message_for_cross_transfer_revert_panic_for_xcall() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);

    let bnusd_amount = 100000u128;

    let data = CrossTransferRevert::new(ctx.withdrawer.clone(), bnusd_amount)
        .encode(&ctx.env, String::from_str(&ctx.env, "xCrossTransferRevert"));
    let decoded = CrossTransferRevert::decode(&ctx.env, data.clone());
    assert_eq!(decoded.to, ctx.withdrawer);

    assert_eq!(client.balance(&ctx.withdrawer), 0);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    let wrong_network_address: String = String::from_str(
        &ctx.env,
        &std::format!(
            "{}/{}",
            "soroban",
            "CBEPDNVYXQGWB5YUBXKJWYJA7OXTZW5LFLNO5JRRGE6Z6C5OSUZPCCEL"
        ),
    );
    client.handle_call_message(&wrong_network_address, &data, &sources);

    assert_eq!(client.balance(&ctx.withdrawer), bnusd_amount as i128)
}

#[test]
fn test_extend_ttl() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();

    ctx.init_context(&client);

    client.extend_ttl()
}

#[test]
fn test_set_upgrade_authority() {
    let ctx = TestContext::default();
    let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
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
