//! This contract demonstrates a sample implementation of the Soroban token
//! interface.
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::balanced_dollar;
use crate::config::ConfigData;
use crate::errors::ContractError;
use crate::metadata::{read_decimal, read_name, read_symbol, write_metadata};
use crate::states::{has_administrator, read_administrator, write_administrator};
use crate::storage_types::{DataKey, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use soroban_sdk::token::{self, Interface as _};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Bytes, BytesN, Env, String, Vec};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;
pub fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

#[contract]
pub struct BalancedDollar;

#[contractimpl]
impl BalancedDollar {
    pub fn initialize(e: Env, admin: Address, config: ConfigData) {
        if has_administrator(&e) {
            panic_with_error!(e, ContractError::ContractAlreadyInitialized)
        }
        write_administrator(&e, &admin);

        //initialize token properties
        let decimal = 18;
        let name = String::from_str(&e, "Balanced Dollar");
        let symbol = String::from_str(&e, "bnUSD");

        if decimal > u8::MAX.into() {
            panic_with_error!(e, ContractError::DecimalMustFitInAu8)
        }

        write_metadata(
            &e,
            TokenMetadata {
                decimal,
                name,
                symbol,
            },
        );
        balanced_dollar::configure(e, config);
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        let admin = read_administrator(&e);
        admin.require_auth();

        balanced_dollar::_mint(&e, to, amount)
    }

    pub fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        write_administrator(&e, &new_admin);
        TokenUtils::new(&e).events().set_admin(admin, new_admin);
    }

    pub fn get_admin(e: Env) -> Address {
        read_administrator(&e)
    }

    pub fn cross_transfer(
        e: Env,
        from: Address,
        amount: u128,
        to: String,
        data: Option<Bytes>
    ) -> Result<(), ContractError> {
        from.require_auth();
        let transfer_data = data.unwrap_or(Bytes::from_array(&e, &[0u8; 32]));
        return balanced_dollar::_cross_transfer(e.clone(), from, amount, to, transfer_data);
    }

    pub fn handle_call_message(
        e: Env,
        from: String,
        data: Bytes,
        protocols: Vec<String>,
    ) -> Result<(), ContractError> {
        return balanced_dollar::_handle_call_message(e, from, data, protocols);
    }

    pub fn is_initialized(e: Env) -> bool {
        has_administrator(&e)
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    pub fn extend_ttl(e: Env) {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    }
}

#[contractimpl]
impl token::Interface for BalancedDollar {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&e, from, spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        check_nonnegative_amount(amount);

        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        TokenUtils::new(&e)
            .events()
            .approve(from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        read_balance(&e, id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount)
    }

    fn burn(e: Env, from: Address, amount: i128) {
        balanced_dollar::_burn(&e, from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount)
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> String {
        read_name(&e)
    }

    fn symbol(e: Env) -> String {
        read_symbol(&e)
    }
}

