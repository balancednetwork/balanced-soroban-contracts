#![cfg(test)]
extern crate std;

use crate::contract::XcallManagerClient;

use soroban_sdk::{
     Vec, String
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
fn test_verify_protocols(){
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);

    ctx.init_context(&client);
    let protocols = Vec::from_array(&ctx.env, [ctx.centralized_connection.to_string()]);
    client.verify_protocols(&protocols);
}

#[test]
fn test_handle_call_message_for_configure_protocols(){
    std::println!("first");
    let ctx = TestContext::default();
    let client = XcallManagerClient::new(&ctx.env, &ctx.registry);
    ctx.env.mock_all_auths();
    std::println!("second");
    ctx.init_context(&client);


    let source_items = [String::from_str(&ctx.env, "sui/address"), String::from_str(&ctx.env, "sui/address1")];
    std::println!("source_items : {:?}", source_items);
    let destination_items = [String::from_str(&ctx.env, "icon/address"), String::from_str(&ctx.env, "icon/address1")];
    let sources = Vec::from_array(&ctx.env, source_items);
    let destinations = Vec::from_array(&ctx.env, destination_items);
    let data = ConfigureProtocols::new(sources.clone(), destinations.clone()).encode(&ctx.env, String::from_str(&ctx.env, "ConfigureProtocols"));
    std::println!("encoded data : {:?}", data);
    let decoded: ConfigureProtocols = ConfigureProtocols::decode(&ctx.env, data.clone());

    std::println!("decoded source : {:?}", decoded.sources);
    assert_eq!(decoded.sources, sources);
    assert_eq!(decoded.destinations, destinations);
    let (s, _) = client.get_protocols();
    client.handle_call_message(&ctx.icon_governance,  &data, &s);

    let (s, d) = client.get_protocols();
    assert_eq!(s, sources);
    assert_eq!(d, destinations);
}