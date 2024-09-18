use crate::balance::{receive_balance, spend_balance};
use soroban_sdk::{xdr::ToXdr, Address, Bytes, Env, String, Vec};
mod xcall {
    soroban_sdk::contractimport!(file = "../../wasm/xcall.wasm");
}

use crate::contract;
use crate::errors::ContractError;
use crate::states::read_administrator;
use crate::{
    config::{get_config, set_config, ConfigData},
    xcall_manager_interface::XcallManagerClient,
};
use soroban_rlp::balanced::address_utils::{get_address_from, is_valid_bytes_address};
use soroban_rlp::balanced::messages::{
    cross_transfer::CrossTransfer, cross_transfer_revert::CrossTransferRevert,
};
use soroban_token_sdk::TokenUtils;
use xcall::{AnyMessage, CallMessageWithRollback, Client, Envelope};
const CROSS_TRANSFER: &str = "xCrossTransfer";
const CROSS_TRANSFER_REVERT: &str = "xCrossTransferRevert";

pub fn configure(env: Env, config: ConfigData) {
    set_config(&env, config);
}

pub fn _cross_transfer(
    e: Env,
    from: Address,
    amount: u128,
    to: String,
    data: Bytes,
) -> Result<(), ContractError> {
    _burn(&e, from.clone(), amount as i128);
    let xcall_message = CrossTransfer::new(from.clone().to_string(), to, amount, data);
    let rollback = CrossTransferRevert::new(from.clone(), amount);
    let config = get_config(&e);
    let icon_bn_usd = config.icon_bn_usd;

    let rollback_bytes = rollback.encode(&e, String::from_str(&e, CROSS_TRANSFER_REVERT));
    let message_bytes = xcall_message.encode(&e, String::from_str(&e, CROSS_TRANSFER));

    let (sources, destinations) = xcall_manager_client(&e, &config.xcall_manager).get_protocols();

    let message = AnyMessage::CallMessageWithRollback(CallMessageWithRollback {
        data: message_bytes,
        rollback: rollback_bytes,
    });
    let envelope: &Envelope = &Envelope {
        message,
        sources,
        destinations,
    };

    let current_address = e.current_contract_address();
    xcall_client(&e, &config.xcall).send_call(&from, &current_address, envelope, &icon_bn_usd);
    Ok(())
}

fn verify_protocol(
    e: &Env,
    xcall_manager: &Address,
    protocols: Vec<String>,
) -> Result<(), ContractError> {
    let verified: bool = xcall_manager_client(e, xcall_manager).verify_protocols(&protocols);
    if !verified {
        return Err(ContractError::ProtocolMismatch);
    }
    Ok(())
}

pub fn _handle_call_message(
    e: Env,
    from: String,
    data: Bytes,
    protocols: Vec<String>,
) -> Result<(), ContractError> {
    let config: ConfigData = get_config(&e);
    let xcall = config.xcall;
    xcall.require_auth();

    let method = CrossTransfer::get_method(&e, data.clone());
    let icon_bn_usd: String = config.icon_bn_usd;
    if method == String::from_str(&e, &CROSS_TRANSFER) {
        if from != icon_bn_usd {
            return Err(ContractError::OnlyIconBnUSD);
        }
        let message = CrossTransfer::decode(&e, data);
        let to_network_address: Address = get_address(message.to, &e)?;
        _mint(&e, to_network_address, message.amount as i128);
    } else if method == String::from_str(&e, &CROSS_TRANSFER_REVERT) {
        if config.xcall_network_address != from {
            return Err(ContractError::OnlyCallService);
        }
        let message = CrossTransferRevert::decode(&e, data);
        _mint(&e, message.to, message.amount as i128);
    } else {
        return Err(ContractError::UnknownMessageType);
    }
    verify_protocol(&e, &config.xcall_manager, protocols)?;
    Ok(())
}

pub fn get_address(network_address: String, env: &Env) -> Result<Address, ContractError> {
    let bytes = network_address.to_xdr(&env);

    if bytes.get(6).unwrap() > 0 {
        return Err(ContractError::InvalidNetworkAddressLength);
    }

    let value_len = bytes.get(7).unwrap();
    let slice = bytes.slice(8..value_len as u32 + 8);
    let mut nid = Bytes::new(&env);
    let mut account = Bytes::new(&env);

    let mut has_seperator = false;
    for (index, value) in slice.clone().iter().enumerate() {
        if has_seperator {
            account.append(&slice.slice(index as u32..slice.len()));
            break;
        } else if value == 47 {
            has_seperator = true;
        } else {
            nid.push_back(value)
        }
    }

    if !has_seperator {
        return Err(ContractError::InvalidNetworkAddress);
    }

    if !is_valid_bytes_address(&account) {
        return Err(ContractError::InvalidAddress);
    }
    Ok(Address::from_string_bytes(&account))
}

fn _mint(e: &Env, to: Address, amount: i128) {
    contract::check_nonnegative_amount(amount);
    let admin: Address = read_administrator(&e);

    receive_balance(e, to.clone(), amount);
    TokenUtils::new(e).events().mint(admin, to, amount);
}

pub fn _burn(e: &Env, from: Address, amount: i128) {
    contract::check_nonnegative_amount(amount);

    spend_balance(e, from.clone(), amount);
    TokenUtils::new(e).events().burn(from, amount);
}

fn xcall_client(e: &Env, xcall: &Address) -> Client<'static> {
    return xcall::Client::new(e, xcall);
}

fn xcall_manager_client(e: &Env, xcall_manager: &Address) -> XcallManagerClient<'static> {
    let client = XcallManagerClient::new(e, xcall_manager);
    return client;
}
