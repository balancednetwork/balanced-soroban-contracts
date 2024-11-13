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
use crate::{token_lib, storage_types};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, Bytes, BytesN, Env, String, Vec,
};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;
pub fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

pub fn initialize(
    e: Env,
    xcall: Address,
    xcall_manager: Address,
    icon_bnusd: String,
    upgrade_auth: Address,
    name: String,
    symbol: String,
    decimal: u8
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

pub fn cross_transfer(
    e: Env,
    from: Address,
    amount: u128,
    to: String,
    data: Option<Bytes>,
) -> Result<(), ContractError> {
    from.require_auth();
    let transfer_data = data.unwrap_or(Bytes::from_array(&e, &[0u8; 32]));
    return token_lib::_cross_transfer(e.clone(), from, amount, to, transfer_data);
}

pub fn handle_call_message(
    e: Env,
    from: String,
    data: Bytes,
    protocols: Vec<String>,
) -> Result<(), ContractError> {
    return token_lib::_handle_call_message(e, from, data, protocols);
}

pub fn is_initialized(e: Env) -> bool {
    storage_types::has_upgrade_auth(&e)
}

pub fn set_upgrade_authority(e: Env, new_upgrade_authority: Address) {
    let upgrade_authority = get_upgrade_authority(&e)?;
    upgrade_authority.require_auth();
    set_upgrade_authority(&e, new_upgrade_authority);
}

pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
    let upgrade_authority = get_upgrade_authority(&e)?;
    upgrade_authority.require_auth();
    e.deployer().update_current_contract_wasm(new_wasm_hash);
}

pub fn extend_ttl(e: Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn allowance(e: Env, from: Address, spender: Address) -> i128 {
    read_allowance(&e, from, spender).amount
}

pub fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
    from.require_auth();

    check_nonnegative_amount(amount);

    write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
    TokenUtils::new(&e)
        .events()
        .approve(from, spender, amount, expiration_ledger);
}

pub fn balance(e: Env, id: Address) -> i128 {
    read_balance(&e, id)
}

pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();

    check_nonnegative_amount(amount);
    spend_balance(&e, from.clone(), amount);
    receive_balance(&e, to.clone(), amount);
    TokenUtils::new(&e).events().transfer(from, to, amount);
}

pub fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
    spender.require_auth();

    check_nonnegative_amount(amount);

    spend_allowance(&e, from.clone(), spender, amount);
    spend_balance(&e, from.clone(), amount);
    receive_balance(&e, to.clone(), amount);
    TokenUtils::new(&e).events().transfer(from, to, amount)
}

pub fn decimals(e: Env) -> u32 {
    read_decimal(&e)
}

pub fn name(e: Env) -> String {
    read_name(&e)
}

pub fn symbol(e: Env) -> String {
    read_symbol(&e)
}

pub fn xcall_manager(e: Env) -> Address {
    storage_types::get_xcall_manager(&e)?
}

pub fn xcall(e: Env) -> Address {
    storage_types::get_xcall(&e)?
}
