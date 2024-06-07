
use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, String, Vec, panic_with_error};
mod xcall {
    soroban_sdk::contractimport!(file = "../../wasm/xcall.wasm");
}
use soroban_rlp::messages::{deposit::Deposit, deposit_revert::DepositRevert, withdraw_to::WithdrawTo};
use crate::{
    config::{get_config, set_config, ConfigData}, 
    states:: {has_registry, read_administrator, read_token_last_current_limit, read_token_last_update, 
        read_token_percentage, read_token_period, write_administrator, write_registry, write_token_current_limit, 
        write_token_last_update, write_token_percentage, write_token_period, extent_ttl, read_tokens, write_tokens},
    storage_types::{DataKey, POINTS}, xcall_manager_interface::XcallManagerClient
};

use crate::errors::ContractError;

use xcall::{AnyMessage, CallMessageWithRollback, Client, Envelope};

const DEPOSIT_NAME: &str = "Deposit";
const WITHDRAW_TO_NAME: &str = "WithdrawTo";
const DEPOSIT_REVERT_NAME: &str = "DepositRevert";

#[contract]
pub struct AssetManager;

#[contractimpl]
impl AssetManager {

    pub fn initialize(env:Env, registry:Address, admin: Address, config: ConfigData) {
        if has_registry(&env.clone()) {
            panic_with_error!(&env, ContractError::ContractAlreadyInitialized)
        }

        write_registry(&env, &registry);
        write_administrator(&env, &admin);
        Self::configure(env, config);
    }
    
    pub fn get_config(env: Env) -> ConfigData {
        get_config(&env)
    }

    pub fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        write_administrator(&e, &new_admin);
    }

    pub fn get_admin(e: Env) -> Address{
        read_administrator(&e)
    }

    pub fn configure(env:Env, config: ConfigData){
        let admin = read_administrator(&env.clone());
        admin.require_auth();

        set_config(&env, config);
    }

    pub fn configure_rate_limit(
        env: Env,
        token: Address,
        period: u128,
        percentage: u128,
    ) {
        let admin = read_administrator(&env.clone());
        admin.require_auth();
        let tokens = read_tokens(&env.clone());
        if tokens.contains(&token) {
            panic_with_error!(&env.clone(), ContractError::TokenExists)
        }else{
            write_tokens(&env, token.clone());
        };

        if percentage > POINTS {
            panic_with_error!(&env.clone(), ContractError::PercentageShouldBeLessThanOrEqualToPOINTS); 
        }
        let token_client = token::Client::new(&env, &token);
        let contract_token_balance = token_client.balance(&env.current_contract_address());
        
        write_token_period(&env.clone(), &token.clone(), period);
        write_token_percentage(&env.clone(), &token.clone(), percentage);
        write_token_last_update(&env.clone(), &token.clone(), env.ledger().timestamp());
        write_token_current_limit(&env.clone(), &token.clone(), (contract_token_balance as u128) * percentage/POINTS);
    }

    pub fn get_rate_limit(env: Env, token: Address ) -> (u128, u128, u64, u128){
        (
            read_token_period(&env, &token.clone()),
            read_token_percentage(&env, &token.clone()),
            read_token_last_update(&env, &token.clone()),
            read_token_last_current_limit(&env, &token.clone()),
        )
    }

    pub fn reset_limit(env: Env, token: Address){
        let balance = Self::get_token_balance(env.clone(), token.clone());
        let percentage: u128 = read_token_percentage(&env, &token.clone());

        env.storage().instance().set(&DataKey::CurrentLimit(token.clone()), &(balance*percentage/POINTS));
    }

    pub fn get_withdraw_limit(env: Env, token: Address) -> Result<u128, ContractError>  {
        let balance = Self::get_token_balance(env.clone(), token.clone());
        return Ok(Self::calculate_limit(env, balance, token)?)
    }

    fn get_token_balance(env: Env, token: Address) -> u128 {
        let token_client = token::Client::new(&env, &token);
        return token_client.balance(&env.current_contract_address()) as u128
    }

    pub fn verify_withdraw(env: Env, token: Address, amount: u128) -> Result<bool, ContractError> {
        let balance = Self::get_token_balance(env.clone(), token.clone());
        let limit = Self::calculate_limit(env.clone(), balance, token.clone())?;
        if balance - amount < limit { panic_with_error!(&env, ContractError::ExceedsWithdrawLimit); };

        write_token_current_limit(&env, &token.clone(), limit);
        write_token_last_update(&env, &token.clone(), env.ledger().timestamp());
        Ok(true)
    }

    pub fn calculate_limit(env: Env, balance: u128, token: Address) -> Result<u128, ContractError> {
        let period: u128 = read_token_period(&env, &token.clone());
        let percentage: u128 =  read_token_percentage(&env, &token.clone());
        if period == 0 {
            return Ok(0);
        }

        let max_limit = (balance * percentage) / POINTS;

        let max_withdraw = balance - max_limit;
        let last_update: u64 = read_token_last_update(&env, &token.clone());
        let time_diff = &env.ledger().timestamp() - last_update;

        let added_allowed_withdrawal = (max_withdraw * u128::from(time_diff)) / period;
        let current_limit: u128 = read_token_last_current_limit(&env, &token.clone());
        let limit: u128 = current_limit - added_allowed_withdrawal;

        let limit = if balance < limit {  balance   } else { limit };
                     
        let final_limit = if limit > max_limit { limit } else { max_limit };
         Ok(final_limit)
    }

    pub fn deposit(
        e: Env,
        from: Address, 
        token: Address,
        amount: u128,
        to: Option<String>,
        data: Option<Bytes>
    ) -> Result<(), ContractError> {
       let deposit_to = to.unwrap_or(String::from_str(&e, ""));
       let deposit_data = data.unwrap_or(Bytes::from_array(&e, &[0u8; 32]));

       Ok(Self::send_deposit_message(e, from, token, amount, deposit_to, deposit_data)?)
    }

    fn send_deposit_message(
        e: Env,
        from: Address,
        token: Address,
        amount: u128,
        to: String,
        data: Bytes,
    ) -> Result<(), ContractError> {
       from.require_auth();
       let current_address = e.clone().current_contract_address();
       Self::transfer_token_to(e.clone(), from.clone(), token.clone(), current_address.clone(), amount);

       let xcall_message: Deposit = Deposit::new(
            token.to_string(),
            from.to_string(),
            to.clone(),
            amount,
            data
        );

        let rollback: DepositRevert = DepositRevert::new(
            token,
            from.clone(),
            amount
        );

        let rollback_bytes = rollback.encode(&e, String::from_str(&e.clone(), DEPOSIT_REVERT_NAME));
        let message_bytes = xcall_message.encode(&e, String::from_str(&e.clone(), DEPOSIT_NAME));
        let (sources, destinations) = Self::xcall_manager(e.clone()).get_protocols();
        let message = AnyMessage::CallMessageWithRollback(CallMessageWithRollback { data: message_bytes, rollback: rollback_bytes });
        let envelope: &Envelope = &Envelope {
            destinations,
            message,
            sources
        };
        let icon_asset_manager = &get_config(&e).icon_asset_manager;
        
        Self::xcall_client(e).send_call(&from, &current_address, envelope, icon_asset_manager);
        Ok(())
    }

    fn xcall_manager(e: Env) -> XcallManagerClient<'static> {
       let client = XcallManagerClient::new(&e, &get_config(&e).xcall_manager);
       return client;
    }

    fn xcall_client(e: Env) -> Client<'static> {
        return xcall::Client::new(&e, &get_config(&e).xcall);
    }

    pub fn handle_call_message(
        e: Env,
        _xcall: Address,
        from: String,
        data: Bytes,
        protocols: Vec<String>
    )  -> Result<(), ContractError>  {
        get_config(&e).xcall.require_auth();
        if !Self::xcall_manager(e.clone()).verify_protocols(&protocols) {
          panic_with_error!(&e, ContractError::ProtocolMismatch);
        };
        let method = Deposit::get_method(&e, data.clone());
        let icon_asset_manager = get_config(&e).icon_asset_manager;
        let current_contract = e.current_contract_address();
        if method == String::from_str(&e, &WITHDRAW_TO_NAME){
            if from != icon_asset_manager{
                panic_with_error!(&e, ContractError::OnlyICONAssetManager);
            };
            let message = WithdrawTo::decode(&e, data);
            Self::withdraw(e, current_contract, Address::from_string(&message.token_address),  Address::from_string(&message.to), message.amount)?;
        } else if method == String::from_str(&e, &DEPOSIT_REVERT_NAME){
            let xcall_network_address = Self::xcall_client(e.clone()).get_network_address();
        
            if from !=  xcall_network_address {
                panic_with_error!(&e, ContractError::OnlyCallService)
            };
            let message: DepositRevert = DepositRevert::decode(&e.clone(), data);
            Self::withdraw(e, current_contract, message.token_address,  message.to, message.amount)?;
        } else {
            panic_with_error!(&e, ContractError::UnknownMessageType);
        }
        Ok(())
    }

    pub fn withdraw(e: Env, from: Address, token: Address, to: Address, amount: u128) -> Result<(), ContractError> {
        if amount <= 0 {
            panic_with_error!(&e, ContractError::AmountIsLessThanMinimumAmount);
        }
        let verified = Self::verify_withdraw(e.clone(), token.clone(), amount)?;
        if verified {
            Self::transfer_token_to(e, from, token, to, amount);
        }
        Ok(())
    }

    fn transfer_token_to(e: Env, from: Address, token: Address, to: Address, amount: u128){
        let token_client = token::Client::new(&e, &token);
        token_client.transfer(&from, &to, &(amount as i128));
    }

    pub fn balance_of(e: Env, token: Address) -> i128 {
        let token_client = token::Client::new(&e, &token);
        return token_client.balance(&e.current_contract_address());
    }

    pub fn has_registry(e: Env) -> bool {
        has_registry(&e)
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    pub fn extend_ttl(e: Env){
        extent_ttl(&e);
    }

}