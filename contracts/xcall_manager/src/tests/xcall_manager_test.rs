#![cfg(test)]
extern crate std;

use crate::contract::XcallManagerClient;

use soroban_sdk::{
    symbol_short, testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Address, IntoVal, String, Vec, vec
};
use soroban_rlp::messages::configure_protocols::ConfigureProtocols;
use super::setup::*;

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
        std::vec![
            (
                ctx.admin.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        ctx.registry.clone(),
                        symbol_short!("set_admin"),
                        (&new_admin,)
                        .into_val(&ctx.env)
                    )),
                    sub_invocations: std::vec![]
                }
            )
        ]
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
fn test_verify_protocols(){
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);

    ctx.init_context(&client);
    let protocols = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.verify_protocols(&protocols);
}

#[test]
fn test_handle_call_message_for_configure_protocols(){
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);


    let source_items = [String::from_str(&ctx.env, "sui/address"), String::from_str(&ctx.env, "sui/address1")];
    let destination_items = [String::from_str(&ctx.env, "icon/address"), String::from_str(&ctx.env, "icon/address1")];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone()).encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());

    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.xcall,&ctx.icon_governance,  &data, &s);

    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}

#[test]
fn test_proposal_removal(){
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    client.propose_removal(&String::from_str(&ctx.env, "sui/address"));
    assert_eq!(String::from_str(&ctx.env, "sui/address"), client.get_proposed_removal())
}

#[test]
fn test_get_modified_proposals(){
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    let source_items = [String::from_str(&ctx.env, "sui/address"), String::from_str(&ctx.env, "sui/address1")];
    let destination_items = [String::from_str(&ctx.env, "icon/address"), String::from_str(&ctx.env, "icon/address1")];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone()).encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));

    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.xcall,&ctx.icon_governance,  &data, &s);

    client.propose_removal(&String::from_str(&ctx.env, "sui/address"));

    let updated_protocal = vec![&ctx.env, String::from_str(&ctx.env, "sui/address1")];
    assert_eq!(updated_protocal, client.get_modified_protocols());
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #12)")]
fn test_get_modified_proposals_panic_no_proposed_removal(){
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    let source_items = [String::from_str(&ctx.env, "sui/address"), String::from_str(&ctx.env, "sui/address1")];
    let destination_items = [String::from_str(&ctx.env, "icon/address"), String::from_str(&ctx.env, "icon/address1")];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone()).encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));

    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.xcall,&ctx.icon_governance,  &data, &s);

    //client.propose_removal(&String::from_str(&ctx.env, "sui/address"));

    let updated_protocal = vec![&ctx.env, String::from_str(&ctx.env, "sui/address1")];
    assert_eq!(updated_protocal, client.get_modified_protocols());
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_handle_call_message_for_configure_protocols_panic_for_only_icon_governance(){
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);


    let source_items = [String::from_str(&ctx.env, "sui/address"), String::from_str(&ctx.env, "sui/address1")];
    let destination_items = [String::from_str(&ctx.env, "icon/address"), String::from_str(&ctx.env, "icon/address1")];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone()).encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());

    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.xcall, &ctx.xcall_network_address,  &data, &s);

    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_handle_call_message_for_configure_protocols_panic_for_protocol_mismatch(){
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);


    let source_items = [String::from_str(&ctx.env, "sui/address"), String::from_str(&ctx.env, "sui/address1")];
    let destination_items = [String::from_str(&ctx.env, "icon/address"), String::from_str(&ctx.env, "icon/address1")];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone()).encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());

    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let s = Vec::from_array(&ctx.env, [ctx.xcall.to_string()]);
    client.handle_call_message(&ctx.xcall,&ctx.icon_governance,  &data, &s);

    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_handle_call_message_for_configure_protocols_panic_for_unknown_mesage_type(){
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);


    let source_items = [String::from_str(&ctx.env, "sui/address"), String::from_str(&ctx.env, "sui/address1")];
    let destination_items = [String::from_str(&ctx.env, "icon/address"), String::from_str(&ctx.env, "icon/address1")];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone()).encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocolsPanic"));
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());

    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let s = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.handle_call_message(&ctx.xcall,&ctx.icon_governance,  &data, &s);

    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}

#[test]
fn test_extend_ttl(){
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    ctx.init_context(&client);

    client.extend_ttl();
}