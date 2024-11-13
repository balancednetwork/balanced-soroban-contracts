//! This contract demonstrates a sample implementation of the Soroban token
//! interface.
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::errors::ContractError;
use crate::metadata::{read_decimal, read_name, read_symbol, write_metadata};
use crate::storage_types::{
    get_upgrade_authority, set_icon_bnusd, set_upgrade_authority, set_xcall, set_xcall_manager,
    INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{panic_with_error, Address, Bytes, BytesN, Env, String, Vec};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;

use crate::storage_types::{self, get_icon_bnusd, get_xcall, get_xcall_manager};
mod xcall {
    soroban_sdk::contractimport!(file = "../../wasm/xcall.wasm");
}

use crate::xcall_manager_interface::XcallManagerClient;
use soroban_rlp::balanced::address_utils::is_valid_bytes_address;
use soroban_rlp::balanced::messages::{
    cross_transfer::CrossTransfer, cross_transfer_revert::CrossTransferRevert,
};
use xcall::{AnyMessage, CallMessageWithRollback, Client, Envelope};
const CROSS_TRANSFER: &str = "xCrossTransfer";
const CROSS_TRANSFER_REVERT: &str = "xCrossTransferRevert";
pub fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}
pub fn _initialize(
    e: Env,
    xcall: Address,
    xcall_manager: Address,
    icon_bnusd: String,
    upgrade_auth: Address,
    name: String,
    symbol: String,
    decimal: u32,
) {
    if storage_types::has_upgrade_auth(&e) {
        panic_with_error!(e, ContractError::ContractAlreadyInitialized)
    }

    write_metadata(
        &e,
        TokenMetadata {
            decimal,
            name,
            symbol,
        },
    );
    set_xcall(&e, xcall);
    set_icon_bnusd(&e, icon_bnusd);
    set_xcall_manager(&e, xcall_manager);
    set_upgrade_authority(&e, upgrade_auth);
}

pub fn _cross_transfer(
    e: Env,
    from: Address,
    amount: u128,
    to: String,
    data: Option<Bytes>,
) -> Result<(), ContractError> {
    from.require_auth();
    let data = data.unwrap_or(Bytes::from_array(&e, &[0u8; 32]));
    if amount <= i128::MAX as u128 {
        _burn(&e, from.clone(), amount as i128);
    } else {
        return Err(ContractError::InvalidAmount);
    }
    let xcall_message = CrossTransfer::new(from.clone().to_string(), to, amount, data);
    let rollback = CrossTransferRevert::new(from.clone(), amount);
    let icon_bn_usd = get_icon_bnusd(&e)?;

    let rollback_bytes = rollback.encode(&e, String::from_str(&e, CROSS_TRANSFER_REVERT));
    let message_bytes = xcall_message.encode(&e, String::from_str(&e, CROSS_TRANSFER));

    let (sources, destinations) = xcall_manager_client(&e, &get_xcall_manager(&e)?).get_protocols();

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
    xcall_client(&e, &get_xcall(&e)?).send_call(&from, &current_address, envelope, &icon_bn_usd);
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
    let xcall = get_xcall(&e)?;
    xcall.require_auth();

    let method = CrossTransfer::get_method(&e, data.clone());
    let icon_bn_usd: String = get_icon_bnusd(&e)?;
    if method == String::from_str(&e, &CROSS_TRANSFER) {
        if from != icon_bn_usd {
            return Err(ContractError::OnlyIconBnUSD);
        }
        let message = CrossTransfer::decode(&e, data);
        let to_network_address: Address = get_address(message.to, &e)?;
        if message.amount <= i128::MAX as u128 {
            _mint(&e, to_network_address, message.amount as i128);
        } else {
            return Err(ContractError::InvalidAmount);
        }
    } else if method == String::from_str(&e, &CROSS_TRANSFER_REVERT) {
        let xcall_network_address = xcall_client(&e, &xcall).get_network_address();
        if xcall_network_address != from {
            return Err(ContractError::OnlyCallService);
        }
        let message = CrossTransferRevert::decode(&e, data);
        if message.amount <= i128::MAX as u128 {
            _mint(&e, message.to, message.amount as i128);
        } else {
            return Err(ContractError::InvalidAmount);
        }
    } else {
        return Err(ContractError::UnknownMessageType);
    }
    verify_protocol(&e, &get_xcall_manager(&e)?, protocols)?;
    Ok(())
}

pub fn _is_initialized(e: Env) -> bool {
    storage_types::has_upgrade_auth(&e)
}

pub fn _set_upgrade_authority(e: Env, new_upgrade_authority: Address) {
    let upgrade_authority = get_upgrade_authority(&e).unwrap();
    upgrade_authority.require_auth();
    set_upgrade_authority(&e, new_upgrade_authority);
}

pub fn _upgrade(e: Env, new_wasm_hash: BytesN<32>) {
    let upgrade_authority = get_upgrade_authority(&e).unwrap();
    upgrade_authority.require_auth();
    e.deployer().update_current_contract_wasm(new_wasm_hash);
}

pub fn _extend_ttl(e: Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn _allowance(e: Env, from: Address, spender: Address) -> i128 {
    read_allowance(&e, from, spender).amount
}

pub fn _approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
    from.require_auth();

    check_nonnegative_amount(amount);

    write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
    TokenUtils::new(&e)
        .events()
        .approve(from, spender, amount, expiration_ledger);
}

pub fn _balance(e: Env, id: Address) -> i128 {
    read_balance(&e, id)
}

pub fn _transfer(e: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();

    check_nonnegative_amount(amount);
    spend_balance(&e, from.clone(), amount);
    receive_balance(&e, to.clone(), amount);
    TokenUtils::new(&e).events().transfer(from, to, amount);
}

pub fn _transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
    spender.require_auth();

    check_nonnegative_amount(amount);

    spend_allowance(&e, from.clone(), spender, amount);
    spend_balance(&e, from.clone(), amount);
    receive_balance(&e, to.clone(), amount);
    TokenUtils::new(&e).events().transfer(from, to, amount)
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
    check_nonnegative_amount(amount);
    let admin = e.current_contract_address();
    receive_balance(e, to.clone(), amount);
    TokenUtils::new(e).events().mint(admin, to, amount);
}

pub fn _burn(e: &Env, from: Address, amount: i128) {
    check_nonnegative_amount(amount);

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

pub fn _decimals(e: Env) -> u32 {
    read_decimal(&e)
}

pub fn _name(e: Env) -> String {
    read_name(&e)
}

pub fn _symbol(e: Env) -> String {
    read_symbol(&e)
}

pub fn _xcall_manager(e: Env) -> Address {
    storage_types::get_xcall_manager(&e).unwrap()
}

pub fn _xcall(e: Env) -> Address {
    storage_types::get_xcall(&e).unwrap()
}
