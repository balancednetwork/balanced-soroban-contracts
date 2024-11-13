#![no_std]

use soroban_sdk::{
    contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec,
};
use spoke_token::{token_lib, errors::ContractError};

#[contract]
pub struct StakedICX;

#[contractimpl]
impl StakedICX {
    pub fn initialize(e: Env, xcall: Address, xcall_manager: Address, icon_bnusd: String, upgrade_auth: Address) {
      
        //initialize token properties
        let decimal = 18;
        let name = String::from_str(&e, "Staked ICX");
        let symbol = String::from_str(&e, "SICX");

        token_lib::_initialize(e.clone(), xcall, xcall_manager, icon_bnusd, upgrade_auth, name, symbol, decimal);
    }

    pub fn cross_transfer(
        e: Env,
        from: Address,
        amount: u128,
        to: String,
        data: Option<Bytes>,
    ) -> Result<(), ContractError> {
        token_lib::_cross_transfer(e, from, amount, to, data)
    }

    pub fn handle_call_message(
        e: Env,
        from: String,
        data: Bytes,
        protocols: Vec<String>,
    ) -> Result<(), ContractError> {
        token_lib::_handle_call_message(e, from, data, protocols)
    }

    pub fn is_initialized(e: Env) -> bool {
        token_lib::_is_initialized(e)
    }

    pub fn set_upgrade_authority(e: Env, new_upgrade_authority: Address) {
        token_lib::_set_upgrade_authority(e, new_upgrade_authority);
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        token_lib::_upgrade(e, new_wasm_hash);
    }

    pub fn extend_ttl(e: Env) {
        token_lib::_extend_ttl(e);
    }

    pub fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        token_lib::_allowance(e, from, spender)
    }

    pub fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        token_lib::_approve(e, from, spender, amount, expiration_ledger);
    }

    pub fn balance(e: Env, id: Address) -> i128 {
        token_lib::_balance(e, id)
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        token_lib::_transfer(e, from, to, amount);
    }

    pub fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        token_lib::_transfer_from(e, spender, from, to, amount);
    }

    pub fn decimals(e: Env) -> u32 {
        token_lib::_decimals(e)
    }

    pub fn name(e: Env) -> String {
        token_lib::_name(e)
    }

    pub fn symbol(e: Env) -> String {
        token_lib::_symbol(e)
    }

    pub fn xcall_manager(e: Env) -> Address {
        token_lib::_xcall_manager(e)
    }

    pub fn xcall(e: Env) -> Address {
        token_lib::_xcall(e)
    }
}
