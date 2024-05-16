use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, String, Vec};

mod xcall {
    soroban_sdk::contractimport!(file = "xcall.wasm");
}

use crate::{
    admin::{read_administrator, write_administrator}, 
    config::{get_config, ConfigData}, 
    messages::{
        deposit::Deposit, 
        deposit_revert :: DepositRevert, 
        withdraw_to::WithdrawTo
    }, 
    states::has_state, 
    storage_types::{DataKey, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, POINTS}, xcall_manager_interface::XcallManagerClient
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

    pub fn initialize(env:Env, registry:Address, admin: Address, xcall: Address, 
        xcall_manager: Address, native_address: Address, icon_asset_manager: String) {
        if has_state(env.clone(), DataKey::Registry) {
            panic!("Contract already initialized.")
        }
        let xcall_network_address = String::from_str(&env, "xcall_network_address"); // xcall::Client::new(&env, &xcall).get_network_address(&icon_asset_manager);
        env.storage().instance().set(&DataKey::Registry, &registry);
        env.storage().instance().set(&DataKey::Admin, &admin);
        Self::configure(env, xcall, xcall_manager, native_address, icon_asset_manager, xcall_network_address );
    }

    pub fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_administrator(&e, &new_admin);
    }

    pub fn get_admin(e: Env) -> Address{
        read_administrator(&e)
    }

    pub fn configure(env:Env, xcall: Address, 
        xcall_manager: Address, native_address: Address, icon_asset_manager: String, xcall_network_address: String){
            let admin = read_administrator(&env.clone());
            admin.require_auth();

            let config: ConfigData = ConfigData { xcall: xcall, xcall_manager: xcall_manager, native_address: native_address, icon_assset_manager: icon_asset_manager, xcall_network_address: xcall_network_address };
            env.storage().instance().set(&DataKey::Config, &config);
    }


    pub fn configure_rate_limit(
        env: Env,
        token: Address,
        period: u128,
        percentage: u128,
    ) {
        let admin = read_administrator(&env.clone());
            admin.require_auth();

        if percentage > POINTS {panic!("Percentage should be less than or equal to POINTS"); }

        let token_client = token::Client::new(&env, &token);
        let contract_token_balance = token_client.balance(&env.current_contract_address());

        env.storage().instance().set(&DataKey::Period(token.clone()), &period);
        env.storage().instance().set(&DataKey::Percentage(token.clone()), &percentage);
        env.storage().instance().set(&DataKey::LastUpdate(token.clone()), &env.ledger().timestamp());
        env.storage().instance().set(&DataKey::CurrentLimit(token.clone()), &(u128::try_from(contract_token_balance).unwrap() * percentage/POINTS));
        
    }

    pub fn reset_limit(env: Env, token: Address){
        let token_client = token::Client::new(&env, &token);
        let contract_token_balance = token_client.balance(&env.current_contract_address());
        let percentage: u128 = env.storage().instance().get(&DataKey::Percentage(token.clone())).unwrap();

        env.storage().instance().set(&DataKey::CurrentLimit(token.clone()), &(u128::try_from(contract_token_balance).unwrap()*percentage/POINTS));
    }

    pub fn verify_withdraw(env: Env, token: Address, amount: u128) {
        let period: u128 = env.storage().instance().get(&DataKey::Period(token.clone())).unwrap();
        let percentage: u128 = env.storage().instance().get(&DataKey::Percentage(token.clone())).unwrap();
        if period == 0 {
            return;
        }

        let token_client = token::Client::new(&env, &token);
        let balance = token_client.balance(&env.current_contract_address());
        let u128_balnace = u128::try_from(balance).unwrap();

        let max_limit = (u128::try_from(balance).unwrap() * percentage) / POINTS;

        let max_withdraw = u128_balnace - max_limit;
        let last_update: u64 = env.storage().instance().get(&DataKey::LastUpdate(token.clone())).unwrap();
        let time_diff = &env.ledger().timestamp() - last_update;

        let added_allowed_withdrawal = (max_withdraw * u128::from(time_diff)) / period;
        let current_limit: u128 = env.storage().instance().get(&DataKey::CurrentLimit(token.clone())).unwrap();
        let limit = current_limit - added_allowed_withdrawal;

        let limit = if u128_balnace < limit {  u128_balnace   } else { limit };
                     
        let final_limit = if limit > max_limit { limit } else { max_limit };
        if u128_balnace - amount < final_limit { panic!("exceeds withdraw limit"); };

        env.storage().instance().set(&DataKey::CurrentLimit(token.clone()), &final_limit);
        env.storage().instance().set(&DataKey::LastUpdate(token.clone()), &env.ledger().timestamp());
    }

    pub fn deposit(
        e: Env,
        from: Address, 
        value: u128,
        token: Address,
        amount: u128,
        to: Option<String>,
        data: Option<Bytes>
    ) {
       let deposit_to = to.unwrap_or(String::from_str(&e, ""));
       let deposit_data = data.unwrap_or(Bytes::from_array(&e, &[0u8; 32]));

        Self::send_deposit_message(e, from, token, amount, deposit_to, deposit_data, value).unwrap();
    }

    pub fn deposit_native(
        e: Env,
        from: Address,
        value: u128,
        amount: u128,
        to: Option<String>,
        data: Option<Bytes>
    )  {
        if value < amount { 
            panic!("Amount less than minimum amount");
        }

        let deposit_to = to.unwrap_or(String::from_str(&e, ""));
        let deposit_data = data.unwrap_or(Bytes::from_array(&e, &[0u8; 32]));

        let fee = value - amount;
        let native_address = get_config(&e).native_address;
        Self::send_deposit_message(e, from, native_address, amount, deposit_to, deposit_data, fee).unwrap();
    }

    fn send_deposit_message(
        e: Env,
        from: Address,
        token: Address,
        amount: u128,
        to: String,
        data: Bytes,
        fee: u128
    ) -> Result<(), ContractError> {
       let current_address = e.clone().current_contract_address();
       Self::transfer_token_to(e.clone(), from.clone(), token.clone(), current_address.clone(), fee);

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
            message,
            sources,
            destinations
        };
        let icon_asset_manager = &get_config(&e).icon_assset_manager;
        Self::xcall_client(e).send_call(&from, &current_address, envelope, icon_asset_manager );
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
    )  {
        get_config(&e).xcall.require_auth();

        if !Self::xcall_manager(e.clone()).verify_protocols(&protocols) {
          panic!("Protocol Mismatch");
        };

        let method = Deposit::get_method(&e, data.clone()).unwrap();

        let icon_asset_manager = get_config(&e).icon_assset_manager;
        let current_contract = e.current_contract_address();
        if method == String::from_str(&e, &WITHDRAW_TO_NAME){
            if from != icon_asset_manager{
                    panic!("onlyICONAssetManager")
            }
            let message = WithdrawTo::decode(&e, data).unwrap();
            
            Self::withdraw(e, current_contract, Address::from_string(&message.token_address),  Address::from_string(&message.to), message.amount);
        } else if method == String::from_str(&e, &DEPOSIT_REVERT_NAME){
            let xcall_network_address = Self::xcall_client(e.clone()).get_network_address();
        
            if from !=  xcall_network_address {
                panic!("onlyCallService")
            };
            let message: DepositRevert = DepositRevert::decode(&e.clone(), data).unwrap();
            Self::withdraw(e, current_contract, message.token_address,  message.to, message.amount);
        } else {
            panic!("Unknown message type");
        }
    }

    pub fn withdraw(e: Env, from: Address, token: Address, to: Address, amount: u128) {
        if amount <= 0 {
            panic!("Amount less than minimum amount");
        }

        Self::verify_withdraw(e.clone(), token.clone(), amount);
        Self::transfer_token_to(e, from, token, to, amount);
    }

    fn transfer_token_to(e: Env, from: Address, token: Address, to: Address, amount: u128){
        let token_client = token::Client::new(&e, &token);
        token_client.transfer_from(&from, &from, &to, &i128::try_from(amount).unwrap());
    }

    pub fn balance_of(e: Env, token: Address) -> i128 {
        let token_client = token::Client::new(&e, &token);
        return token_client.balance(&e.current_contract_address());
    }


    pub fn has_registry(e: Env) -> bool {
        has_state(e, DataKey::Registry)
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

}