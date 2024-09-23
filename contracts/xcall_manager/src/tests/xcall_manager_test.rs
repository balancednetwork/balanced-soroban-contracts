#![cfg(test)]
extern crate std;

use crate::{config, contract::XcallManagerClient};

use super::setup::*;
use soroban_rlp::balanced::messages::configure_protocols::ConfigureProtocols;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, IntoVal, String, Symbol, Vec,
};

#[test]
fn test_initialize() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);

    ctx.init_context(&client);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    let destinations = Vec::from_array(&ctx.env, [String::from_str(&ctx.env, "icon/address")]);
    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}

#[test]
fn test_set_admin() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
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
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_initialize_panic_already_initialized() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);

    ctx.init_context(&client);
    ctx.init_context(&client);

    let sources = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    let destinations = Vec::from_array(&ctx.env, [String::from_str(&ctx.env, "icon/address")]);
    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}

#[test]
fn test_whitelist_action() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);

    ctx.env.mock_all_auths();
    ctx.init_context(&client);
    let source_items = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let destination_items = [
        String::from_str(&ctx.env, "icon/address"),
        String::from_str(&ctx.env, "icon/address1"),
    ];

    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone())
        .encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    client.white_list_actions(&data);

    let result = client.remove_action(&data);
    assert!(result == true)
}

#[test]
fn test_verify_protocols() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);

    ctx.init_context(&client);
    let protocols = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.verify_protocols(&protocols);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #13)")]
fn test_handle_call_message_for_configure_protocols_panic_for_action_not_whitelisted() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    let source_items = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let destination_items = [
        String::from_str(&ctx.env, "icon/address"),
        String::from_str(&ctx.env, "icon/address1"),
    ];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone())
        .encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());
    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.icon_governance, &data, &s);

    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}

#[test]
fn test_handle_call_message_for_configure_protocols() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    let source_items = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let destination_items = [
        String::from_str(&ctx.env, "icon/address"),
        String::from_str(&ctx.env, "icon/address1"),
    ];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone())
        .encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());
    client.white_list_actions(&data);
    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.icon_governance, &data, &s);

    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);

    //verify multiple protocols
    let wrong_sources = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address"),
    ];
    let verifiy_false = client.verify_protocols(&Vec::from_array(&ctx.env, wrong_sources));
    assert_eq!(verifiy_false, false);

    let wrong_sources_second = [
        String::from_str(&ctx.env, "stellar/address1"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let verifiy_false_second =
        client.verify_protocols(&Vec::from_array(&ctx.env, wrong_sources_second));
    assert_eq!(verifiy_false_second, false);

    let correct_sources = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let verifiy_true = client.verify_protocols(&Vec::from_array(&ctx.env, correct_sources));
    assert_eq!(verifiy_true, true);

    let correct_sources_second: [String; 2] = [
        String::from_str(&ctx.env, "stellar/address1"),
        String::from_str(&ctx.env, "stellar/address"),
    ];
    let verifiy_true = client.verify_protocols(&Vec::from_array(&ctx.env, correct_sources_second));
    assert_eq!(verifiy_true, true);

    //verify protocol recovery
    client.propose_removal(&String::from_str(&ctx.env, "stellar/address1"));
    let with_protocol_remove: [String; 1] = [String::from_str(&ctx.env, "stellar/address")];
    client.verify_protocol_recovery(&Vec::from_array(&ctx.env, with_protocol_remove));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #12)")]
fn test_verify_protocol_recovery_without_removing_protocol() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    let source_items = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let destination_items = [
        String::from_str(&ctx.env, "icon/address"),
        String::from_str(&ctx.env, "icon/address1"),
    ];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone())
        .encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());
    client.white_list_actions(&data);
    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.icon_governance, &data, &s);

    //verify protocol recovery
    let without_protocol_remove: [String; 2] = [
        String::from_str(&ctx.env, "stellar/address1"),
        String::from_str(&ctx.env, "stellar/address"),
    ];
    client.verify_protocol_recovery(&Vec::from_array(&ctx.env, without_protocol_remove));
}

#[test]
fn test_proposal_removal() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    client.propose_removal(&String::from_str(&ctx.env, "stellar/address"));
    assert_eq!(
        String::from_str(&ctx.env, "stellar/address"),
        client.get_proposed_removal()
    )
}

#[test]
fn test_get_modified_proposals() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    let source_items = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let destination_items = [
        String::from_str(&ctx.env, "icon/address"),
        String::from_str(&ctx.env, "icon/address1"),
    ];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone())
        .encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    client.white_list_actions(&data);
    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.icon_governance, &data, &s);

    client.propose_removal(&String::from_str(&ctx.env, "stellar/address"));

    let updated_protocal = vec![&ctx.env, String::from_str(&ctx.env, "stellar/address1")];
    assert_eq!(updated_protocal, client.get_modified_protocols());
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #12)")]
fn test_get_modified_proposals_panic_no_proposed_removal() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    let source_items = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let destination_items = [
        String::from_str(&ctx.env, "icon/address"),
        String::from_str(&ctx.env, "icon/address1"),
    ];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone())
        .encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    client.white_list_actions(&data);
    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.icon_governance, &data, &s);

    let updated_protocal = vec![&ctx.env, String::from_str(&ctx.env, "stellar/address1")];
    assert_eq!(updated_protocal, client.get_modified_protocols());
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_handle_call_message_for_configure_protocols_panic_for_only_icon_governance() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    let source_items = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let destination_items = [
        String::from_str(&ctx.env, "icon/address"),
        String::from_str(&ctx.env, "icon/address1"),
    ];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone())
        .encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());
    client.white_list_actions(&data);
    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.xcall_network_address, &data, &s);

    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_handle_call_message_for_configure_protocols_panic_for_protocol_mismatch() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    let source_items = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let destination_items = [
        String::from_str(&ctx.env, "icon/address"),
        String::from_str(&ctx.env, "icon/address1"),
    ];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone())
        .encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());
    client.white_list_actions(&data);
    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let s = Vec::from_array(&ctx.env, [ctx.xcall.to_string()]);
    client.handle_call_message(&ctx.icon_governance, &data, &s);

    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_handle_call_message_for_configure_protocols_panic_for_unknown_mesage_type() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    let source_items = [
        String::from_str(&ctx.env, "stellar/address"),
        String::from_str(&ctx.env, "stellar/address1"),
    ];
    let destination_items = [
        String::from_str(&ctx.env, "icon/address"),
        String::from_str(&ctx.env, "icon/address1"),
    ];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone()).encode(
        &ctx.env,
        String::from_str(&ctx.env, "ConfigureProtocolsPanic"),
    );
    client.white_list_actions(&data);
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());

    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let s = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.icon_governance, &data, &s);

    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}

#[test]
fn test_extend_ttl() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    client.extend_ttl();
}

#[test]
fn test_set_upgrade_authority() {
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
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
