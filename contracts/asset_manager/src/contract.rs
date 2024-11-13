use soroban_sdk::{
    contract, contractimpl, token, Address, Bytes, BytesN, Env, String, Vec,
};
mod xcall {
    soroban_sdk::contractimport!(file = "../../wasm/xcall.wasm");
}
use crate::errors::ContractError;
use crate::states::{self};
use crate::storage_types::{ConfigData, RateLimit};
use crate::{
    states::{
        extent_ttl, has_upgrade_authority, read_administrator,write_administrator,
    },
    storage_types::POINTS,
    xcall_manager_interface::XcallManagerClient,
};
use soroban_rlp::balanced::address_utils::is_valid_string_address;
use soroban_rlp::balanced::messages::{
    deposit::Deposit, deposit_revert::DepositRevert, withdraw_to::WithdrawTo,
};

use xcall::{AnyMessage, CallMessageWithRollback, Client, Envelope};

const DEPOSIT_NAME: &str = "Deposit";
const WITHDRAW_TO_NAME: &str = "WithdrawTo";
const DEPOSIT_REVERT_NAME: &str = "DepositRevert";

#[contract]
pub struct AssetManager;

#[contractimpl]
impl AssetManager {
    pub fn initialize(env: Env, admin: Address, xcall: Address, xcall_manager: Address, native_address: Address, icon_asset_manager: String, upgrade_authority: Address)  -> Result<(), ContractError> {
        if has_upgrade_authority(&env.clone()) {
            return Err(ContractError::ContractAlreadyInitialized)
        }
        write_administrator(&env, &admin);
        states::write_xcall(&env, xcall);
        states::write_xcall_manager(&env, xcall_manager);
        states::write_icon_asset_manager(&env, icon_asset_manager);
        states::write_upgrade_authority(&env, upgrade_authority);
        states::write_native_address(&env, native_address);
        Ok(())
    }

    pub fn get_config(env: Env) -> ConfigData {
        ConfigData {
            xcall: states::read_xcall(&env),
            xcall_manager: states::read_xcall_manager(&env),
            native_address: states::read_native_address(&env),
            icon_asset_manager: states::read_icon_asset_manager(&env),
            upgrade_authority: states::read_upgrade_authority(&env),
        }
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        let admin = read_administrator(&env);
        admin.require_auth();

        write_administrator(&env, &new_admin);
    }

    pub fn get_admin(env: Env) -> Address {
        read_administrator(&env)
    }

    pub fn configure_rate_limit(
        env: Env,
        token_address: Address,
        period: u64,
        percentage: u32,
    ) -> Result<(), ContractError> {
        let admin = read_administrator(&env);
        admin.require_auth();
        if percentage > POINTS as u32 {
            return Err(ContractError::PercentageShouldBeLessThanOrEqualToPOINTS);
        }
        states::write_period(&env, token_address.clone(), period);
        states::write_percentage(&env, token_address.clone(), percentage);
        states::write_last_update(&env, token_address.clone(), env.ledger().timestamp());
        states::write_current_limit(&env, token_address, 0);
        Ok(())
    }

    pub fn get_rate_limit(env: Env, token_address: Address) -> RateLimit {
        RateLimit {
            period: states::read_period(&env, token_address.clone()),
            percentage: states::read_percentage(&env, token_address.clone()),
            last_update: states::read_last_update(&env, token_address.clone()),
            current_limit: states::read_current_limit(&env, token_address),
        }
    }

    pub fn reset_limit(env: Env, token: Address) -> Result<bool, ContractError> {
        let admin = read_administrator(&env);
        admin.require_auth();
        let balance = Self::get_token_balance(&env, token.clone());
        let percentage = states::read_percentage(&env, token.clone());
        let current_limit = (balance * percentage as u128 / POINTS) as u64;
        states::write_current_limit(&env, token, current_limit);
        Ok(true)
    }

    pub fn get_withdraw_limit(env: Env, token: Address) -> Result<u128, ContractError> {
        let balance = Self::get_token_balance(&env, token.clone());
        return Ok(Self::calculate_limit(&env, balance, token)?);
    }

    fn get_token_balance(env: &Env, token: Address) -> u128 {
        let token_client = token::Client::new(env, &token);
        return token_client.balance(&env.current_contract_address()) as u128;
    }

    fn verify_withdraw(env: Env, token: Address, amount: u128) -> Result<bool, ContractError> {
        let balance = Self::get_token_balance(&env, token.clone());
        let limit = Self::calculate_limit(&env, balance, token.clone())?;
        if balance - amount < limit {
            return Err(ContractError::ExceedsWithdrawLimit);
        };

        states::write_current_limit(&env, token.clone(), limit as u64);
        states::write_last_update(&env, token.clone(), env.ledger().timestamp());
        Ok(true)
    }

    pub fn calculate_limit(
        env: &Env,
        balance: u128,
        token: Address,
    ) -> Result<u128, ContractError> {
        let period: u128 = states::read_period(&env, token.clone()) as u128;
        let percentage: u128 = states::read_percentage(&env, token.clone()) as u128;
        let last_update: u64 = states::read_last_update(&env, token.clone());
        let current_limit: u64 = states::read_current_limit(&env, token.clone());
        
        if period == 0 {
            return Ok(0);
        }

        let min_reserve = (balance * percentage) / POINTS;

        let max_withdraw = balance - min_reserve;
        let time_diff = &env.ledger().timestamp() - last_update;

        let allowed_withdrawal = (max_withdraw * time_diff as u128) / period;
        let mut reserve: u128 = current_limit as u128;

        if reserve > allowed_withdrawal {
            reserve = reserve - allowed_withdrawal;
        }

        let reserve = if reserve > min_reserve {
            reserve
        } else {
            min_reserve
        };
        Ok(reserve)
    }

    pub fn deposit(
        e: Env,
        from: Address,
        token: Address,
        amount: u128,
        to: Option<String>,
        data: Option<Bytes>,
    ) -> Result<(), ContractError> {
        if amount <= 0{
            return Err(ContractError::AmountIsLessThanMinimumAmount);
        }

        let deposit_to = to.unwrap_or(String::from_str(&e, ""));
        let deposit_data = data.unwrap_or(Bytes::from_array(&e, &[0u8; 32]));

        Ok(Self::send_deposit_message(
            e,
            from,
            token,
            amount,
            deposit_to,
            deposit_data,
        )?)
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
        let current_address = e.current_contract_address();
        Self::transfer_token_to(
            &e,
            from.clone(),
            token.clone(),
            current_address.clone(),
            amount,
        )?;

        let xcall_message: Deposit = Deposit::new(
            token.to_string(),
            from.to_string(),
            to.clone(),
            amount,
            data,
        );

        let xcall = states::read_xcall(&e);
        let xcall_manager = states::read_xcall_manager(&e);
        let icon_asset_manager = states::read_icon_asset_manager(&e);

        let rollback: DepositRevert = DepositRevert::new(token, from.clone(), amount);
        let rollback_bytes = rollback.encode(&e, String::from_str(&e, DEPOSIT_REVERT_NAME));
        let message_bytes = xcall_message.encode(&e, String::from_str(&e, DEPOSIT_NAME));
        let (sources, destinations) =
            Self::xcall_manager(&e, &xcall_manager).get_protocols();
        let message = AnyMessage::CallMessageWithRollback(CallMessageWithRollback {
            data: message_bytes,
            rollback: rollback_bytes,
        });
        let envelope: &Envelope = &Envelope {
            destinations,
            message,
            sources,
        };

        Self::xcall_client(&e, &xcall).send_call(
            &from,
            &current_address,
            envelope,
            &icon_asset_manager,
        );
        Ok(())
    }

    fn xcall_manager(e: &Env, xcall_manager: &Address) -> XcallManagerClient<'static> {
        let client = XcallManagerClient::new(e, xcall_manager);
        return client;
    }

    fn xcall_client(e: &Env, xcall: &Address) -> Client<'static> {
        return xcall::Client::new(e, xcall);
    }

    pub fn handle_call_message(
        e: Env,
        from: String,
        data: Bytes,
        protocols: Vec<String>,
    ) -> Result<(), ContractError> {
        let xcall = states::read_xcall(&e);
        let xcall_manager = states::read_xcall_manager(&e);
        let icon_asset_manager = states::read_icon_asset_manager(&e);        
        
        xcall.require_auth();

        let method = Deposit::get_method(&e, data.clone());

        let current_contract = e.current_contract_address();
        if method == String::from_str(&e, &WITHDRAW_TO_NAME) {
            if from != icon_asset_manager {
                return Err(ContractError::OnlyICONAssetManager);
            }
            let message = WithdrawTo::decode(&e, data);
            if !is_valid_string_address(&message.to)
                || !is_valid_string_address(&message.token_address)
            {
                return Err(ContractError::InvalidAddress);
            }
            Self::withdraw(
                &e,
                current_contract,
                Address::from_string(&message.token_address),
                Address::from_string(&message.to),
                message.amount,
            )?;
        } else if method == String::from_str(&e, &DEPOSIT_REVERT_NAME) {
            let xcall_network_address = Self::xcall_client(&e, &xcall).get_network_address();
            if xcall_network_address != from {
                return Err(ContractError::OnlyCallService);
            }
            let message: DepositRevert = DepositRevert::decode(&e.clone(), data);
            Self::withdraw(
                &e,
                current_contract,
                message.token_address,
                message.to,
                message.amount,
            )?;
        } else {
            return Err(ContractError::UnknownMessageType);
        }
        if !Self::xcall_manager(&e, &xcall_manager).verify_protocols(&protocols) {
            return Err(ContractError::ProtocolMismatch);
        }
        Ok(())
    }

    fn withdraw(
        e: &Env,
        from: Address,
        token: Address,
        to: Address,
        amount: u128,
    ) -> Result<(), ContractError> {
        if amount <= 0 {
            return Err(ContractError::AmountIsLessThanMinimumAmount);
        }

        let verified = Self::verify_withdraw(e.clone(), token.clone(), amount)?;
        if verified {
            Self::transfer_token_to(e, from, token, to, amount)?;
        }
        Ok(())
    }

    fn transfer_token_to(e: &Env, from: Address, token: Address, to: Address, amount: u128) -> Result<(), ContractError> {
        let token_client = token::Client::new(e, &token);
        if amount <= i128::MAX as u128 {
            token_client.transfer(&from, &to, &(amount as i128));
         } else {
             return Err(ContractError::InvalidAmount)
         }
        Ok(())
    }

    pub fn balance_of(e: Env, token: Address) -> i128 {
        let token_client = token::Client::new(&e, &token);
        return token_client.balance(&e.current_contract_address());
    }

    pub fn is_initialized(e: Env) -> bool {
        states::has_upgrade_authority(&e)
    }

    pub fn set_upgrade_authority(e: Env, new_upgrade_authority: Address) {
        let upgrade_authority = states::read_upgrade_authority(&e);
        upgrade_authority.require_auth();

        states::write_upgrade_authority(&e, new_upgrade_authority);
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let upgrade_authority = states::read_upgrade_authority(&e);
        upgrade_authority.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    pub fn extend_ttl(e: Env) {
        extent_ttl(&e);
    }
}
