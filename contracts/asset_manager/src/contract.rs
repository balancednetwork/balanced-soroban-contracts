use soroban_sdk::{
    contract, contractimpl, panic_with_error, token, Address, Bytes, BytesN, Env, String, Vec,
};
mod xcall {
    soroban_sdk::contractimport!(file = "../../wasm/xcall.wasm");
}
use crate::errors::ContractError;
use crate::storage_types::TokenData;
use crate::{
    config::{self, get_config, set_config, ConfigData},
    states::{
        extent_ttl, has_registry, read_administrator, read_token_data, read_tokens,
        write_administrator, write_registry, write_token_data, write_tokens,
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
    pub fn initialize(env: Env, registry: Address, admin: Address, config: ConfigData) {
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

    pub fn set_admin(env: Env, new_admin: Address) {
        let admin = read_administrator(&env);
        admin.require_auth();

        write_administrator(&env, &new_admin);
    }

    pub fn get_admin(env: Env) -> Address {
        read_administrator(&env)
    }

    pub fn configure(env: Env, config: ConfigData) {
        let admin = read_administrator(&env);
        admin.require_auth();

        set_config(&env, config);
    }

    pub fn configure_rate_limit(
        env: Env,
        token_address: Address,
        period: u64,
        percentage: u32,
    ) -> Result<(), ContractError> {
        let admin = read_administrator(&env);
        admin.require_auth();
        let tokens = read_tokens(&env);
        if tokens.contains(&token_address) {
            return Err(ContractError::TokenExists);
        } else {
            write_tokens(&env, token_address.clone());
        }

        if percentage > POINTS as u32 {
            return Err(ContractError::PercentageShouldBeLessThanOrEqualToPOINTS);
        }

        write_token_data(
            &env,
            token_address,
            TokenData {
                period,
                percentage,
                last_update: env.ledger().timestamp(),
                current_limit: 0,
            },
        );
        Ok(())
    }

    pub fn get_rate_limit(env: Env, token_address: Address) -> (u64, u32, u64, u64) {
        let data: TokenData = read_token_data(&env, token_address);
        (
            data.period,
            data.percentage,
            data.last_update,
            data.current_limit,
        )
    }

    pub fn reset_limit(env: Env, token: Address) {
        let balance = Self::get_token_balance(&env, token.clone());
        let mut data: TokenData = read_token_data(&env, token.clone());
        data.current_limit = (balance * data.percentage as u128 / POINTS) as u64;
        write_token_data(&env, token, data);
    }

    pub fn get_withdraw_limit(env: Env, token: Address) -> Result<u128, ContractError> {
        let balance = Self::get_token_balance(&env, token.clone());
        return Ok(Self::calculate_limit(&env, balance, token)?);
    }

    fn get_token_balance(env: &Env, token: Address) -> u128 {
        let token_client = token::Client::new(env, &token);
        return token_client.balance(&env.current_contract_address()) as u128;
    }

    pub fn verify_withdraw(env: Env, token: Address, amount: u128) -> Result<bool, ContractError> {
        let balance = Self::get_token_balance(&env, token.clone());
        let limit = Self::calculate_limit(&env, balance, token.clone())?;
        if balance - amount < limit {
            panic_with_error!(&env, ContractError::ExceedsWithdrawLimit);
        };
        let mut data: TokenData = read_token_data(&env, token.clone());
        data.current_limit = limit as u64;
        data.last_update = env.ledger().timestamp();
        write_token_data(&env, token, data);
        Ok(true)
    }

    pub fn calculate_limit(
        env: &Env,
        balance: u128,
        token: Address,
    ) -> Result<u128, ContractError> {
        let data: TokenData = read_token_data(&env, token);
        let period: u128 = data.period as u128;
        let percentage: u128 = data.percentage as u128;
        if period == 0 {
            return Ok(0);
        }

        let min_reserve = (balance * percentage) / POINTS;

        let max_withdraw = balance - min_reserve;
        let last_update: u64 = data.last_update;
        let time_diff = (&env.ledger().timestamp() - last_update) / 1000;

        let allowed_withdrawal = (max_withdraw * time_diff as u128) / period;
        let mut reserve: u128 = data.current_limit as u128;

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
        );

        let xcall_message: Deposit = Deposit::new(
            token.to_string(),
            from.to_string(),
            to.clone(),
            amount,
            data,
        );

        let rollback: DepositRevert = DepositRevert::new(token, from.clone(), amount);
        let config = get_config(&e);
        let rollback_bytes = rollback.encode(&e, String::from_str(&e, DEPOSIT_REVERT_NAME));
        let message_bytes = xcall_message.encode(&e, String::from_str(&e, DEPOSIT_NAME));
        let (sources, destinations) =
            Self::xcall_manager(&e, &config.xcall_manager).get_protocols();
        let message = AnyMessage::CallMessageWithRollback(CallMessageWithRollback {
            data: message_bytes,
            rollback: rollback_bytes,
        });
        let envelope: &Envelope = &Envelope {
            destinations,
            message,
            sources,
        };

        Self::xcall_client(&e, &config.xcall).send_call(
            &from,
            &current_address,
            envelope,
            &config.icon_asset_manager,
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
        let config = get_config(&e);
        let xcall = config.xcall;
        xcall.require_auth();

        let method = Deposit::get_method(&e, data.clone());

        let icon_asset_manager = config.icon_asset_manager;
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
            if config.xcall_network_address != from {
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
        if !Self::xcall_manager(&e, &config.xcall_manager).verify_protocols(&protocols) {
            return Err(ContractError::ProtocolMismatch);
        }
        Ok(())
    }

    pub fn withdraw(
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
            Self::transfer_token_to(e, from, token, to, amount);
        }
        Ok(())
    }

    fn transfer_token_to(e: &Env, from: Address, token: Address, to: Address, amount: u128) {
        let token_client = token::Client::new(e, &token);
        token_client.transfer(&from, &to, &(amount as i128));
    }

    pub fn balance_of(e: Env, token: Address) -> i128 {
        let token_client = token::Client::new(&e, &token);
        return token_client.balance(&e.current_contract_address());
    }

    pub fn has_registry(e: Env) -> bool {
        has_registry(&e)
    }

    pub fn set_upgrade_authority(e: Env, upgrade_authority: Address) {
        let mut config = config::get_config(&e);

        config.upgrade_authority.require_auth();

        config.upgrade_authority = upgrade_authority;
        config::set_config(&e, config);
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let config = get_config(&e);
        config.upgrade_authority.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    pub fn extend_ttl(e: Env) {
        extent_ttl(&e);
    }
}
