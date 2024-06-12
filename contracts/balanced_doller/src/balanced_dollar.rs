use soroban_sdk::{panic_with_error, Address, Bytes, Env, String, Vec, xdr::ToXdr};
use crate::balance::{spend_balance, receive_balance};
mod xcall {
    soroban_sdk::contractimport!(file = "../../wasm/xcall.wasm");
}

use soroban_rlp::messages::{cross_transfer::CrossTransfer, cross_transfer_revert::CrossTransferRevert};
use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use crate::states::read_administrator;
use crate::{
     config::{ get_config, set_config, ConfigData}, xcall_manager_interface::XcallManagerClient
};

use crate::errors::ContractError;

use xcall::{AnyMessage, CallMessageWithRollback, Client, Envelope};
use soroban_token_sdk::TokenUtils;
use crate::contract;

const CROSS_TRANSFER: &str = "xCrossTransfer";
const CROSS_TRANSFER_REVERT: &str = "xCrossTransferRevert";
    
pub fn configure(env:Env, config: ConfigData){
    set_config(&env, config);
}

pub fn _cross_transfer(
    e: Env,
    from: Address,
    amount: u128,
    to: String,
    data: Bytes
)  -> Result<(), ContractError> {
    _burn(e.clone(), from.clone(), amount as i128);
    let xcall_message = CrossTransfer::new(
        from.clone().to_string(),
        to,
        amount,
        data
    );

    let rollback = CrossTransferRevert::new(
        from.clone(),
        amount
    );

    let rollback_bytes = rollback.encode(&e, String::from_str(&e.clone(), CROSS_TRANSFER_REVERT));
    let message_bytes = xcall_message.encode(&e, String::from_str(&e.clone(), CROSS_TRANSFER));
    let (sources, destinations) =  xcall_manager_client(e.clone()).get_protocols();

    let message = AnyMessage::CallMessageWithRollback(CallMessageWithRollback { data: message_bytes, rollback: rollback_bytes });
    let envelope: &Envelope = &Envelope {
        message,
        sources,
        destinations
    };
    let icon_bn_usd = &get_config(&e).icon_bn_usd;
    let current_address = e.clone().current_contract_address();
    xcall_client(e).send_call(&from, &current_address, envelope, icon_bn_usd );
    Ok(())

}


pub fn _handle_call_message(
    e: Env,
    from: String,
    data: Bytes,
    protocols: Vec<String>
) {
    let xcall = get_config(&e).xcall;
    xcall.require_auth();
    if !xcall_manager_client(e.clone()).verify_protocols(&protocols) {
        panic_with_error!(e, ContractError::ProtocolMismatch)
    };

    let method = CrossTransfer::get_method(&e, data.clone());
    let icon_bn_usd = get_config(&e).icon_bn_usd;
    if method == String::from_str(&e, &CROSS_TRANSFER){
        if from!=icon_bn_usd {
            panic_with_error!(e, ContractError::OnlyIconBnUSD)
        }
        let message = CrossTransfer::decode(&e.clone(), data);
        let to_network_address = get_address(message.to.clone(), &e.clone());
        _mint(e.clone(), to_network_address, message.amount as i128 );
    } else if method == String::from_str(&e, &CROSS_TRANSFER_REVERT){
        let xcall_network_address = self::xcall_client(e.clone()).get_network_address();
        if from!=xcall_network_address {
            panic_with_error!(e, ContractError::OnlyCallService)
        }
        let message = CrossTransferRevert::decode(&e.clone(), data);
        _mint(e.clone(), message.to, message.amount as i128);
    }else{
        panic_with_error!(e, ContractError::UnknownMessageType)
    }

}

pub fn get_address(network_address: String, env: &Env) -> Address {
    let bytes = network_address.to_xdr(&env);

    if bytes.get(6).unwrap() > 0 {
        panic!("Invalid network address length")
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
        panic!("Invalid network address")
    }

    Address::from_string_bytes(&account)
}

pub fn _mint(e: Env, to: Address, amount: i128) {
    contract::check_nonnegative_amount(amount);
    let admin: Address = read_administrator(&e);

    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    receive_balance(&e, to.clone(), amount);
    TokenUtils::new(&e).events().mint(admin, to, amount);
}

pub fn _burn(e: Env, from: Address, amount: i128) {
    contract::check_nonnegative_amount(amount);

    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    spend_balance(&e, from.clone(), amount);
    TokenUtils::new(&e).events().burn(from, amount);
}


fn xcall_client(e: Env) -> Client<'static> {
    return xcall::Client::new(&e, &get_config(&e).xcall);
}

fn xcall_manager_client(e: Env) -> XcallManagerClient<'static> {
    let client = XcallManagerClient::new(&e, &get_config(&e).xcall_manager);
    return client;
}
