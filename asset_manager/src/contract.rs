use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, vec, Address, bytes, BytesN, Env, String, Symbol, Vec, Map};

use crate::{
    admin:: {has_administrator, read_administrator, write_administrator},
    storage_types::{DataKey, POINTS, INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT},
    states::{has_state, read_address_state, read_string_state}
};

use crate::contract::xcall_manager::Client;

pub mod xcall_manager {
    soroban_sdk::contractimport!(
        file = "../xcall_manager/target/wasm32-unknown-unknown/release/xcall_manager.wasm"
    );
}

pub mod xcall {
    soroban_sdk::contractimport!(
        file = "../xcall/target/wasm32-unknown-unknown/release/xcall.wasm"
    );
}

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

    pub fn configure(env:Env, xcall: Address, 
        xcall_manager: Address, native_address: Address, icon_asset_manager: String, xcall_network_address: String){
            let admin = read_administrator(&env.clone());
            admin.require_auth();
            
            env.storage().instance().set(&DataKey::XCall, &xcall);
            env.storage().instance().set(&DataKey::XCallManager, &xcall_manager);
            env.storage().instance().set(&DataKey::NativeAddress, &native_address);
            env.storage().instance().set(&DataKey::IconAssetManager, &icon_asset_manager);
            env.storage().instance().set(&DataKey::XCallNetworkAddress, &xcall_network_address);
    }


    pub fn configure_rate_limit(
        env: Env,
        token: Address,
        period: i128,
        percentage: i128,
    ) {
        let admin = read_administrator(&env.clone());
            admin.require_auth();

        if percentage > POINTS {panic!("Percentage should be less than or equal to POINTS"); }

        let token_client = token::Client::new(&env, &token);
        let contract_token_balance = token_client.balance(&env.current_contract_address());

        env.storage().instance().set(&DataKey::Period(token.clone()), &period);
        env.storage().instance().set(&DataKey::Percentage(token.clone()), &percentage);
        env.storage().instance().set(&DataKey::LastUpdate(token.clone()), &env.ledger().timestamp());
        env.storage().instance().set(&DataKey::CurrentLimit(token.clone()), &(contract_token_balance*percentage/POINTS));
        
    }

    pub fn reset_limit(env: Env, token: Address){
        let token_client = token::Client::new(&env, &token);
        let contract_token_balance = token_client.balance(&env.current_contract_address());
        let percentage: i128 = env.storage().instance().get(&DataKey::Percentage(token.clone())).unwrap();

        env.storage().instance().set(&DataKey::CurrentLimit(token.clone()), &(contract_token_balance*percentage/POINTS));
    }

    pub fn verify_withdraw(env: Env, token: Address, amount: i128) {
        let period: i128 = env.storage().instance().get(&DataKey::Period(token.clone())).unwrap();
        let percentage: i128 = env.storage().instance().get(&DataKey::Percentage(token.clone())).unwrap();
        if period == 0 {
            return;
        }

        let token_client = token::Client::new(&env, &token);
        let balance = token_client.balance(&env.current_contract_address());

        let max_limit = (balance * percentage) / POINTS;

        // The maximum amount that can be withdraw in one period
        let max_withdraw = balance - max_limit;
        let last_update: u64 = env.storage().instance().get(&DataKey::LastUpdate(token.clone())).unwrap();
        let time_diff = &env.ledger().timestamp() - last_update;

        // The amount that should be added as availbe
        let added_allowed_withdrawal = (max_withdraw * i128::from(time_diff)) / period;
        let current_limit: i128 = env.storage().instance().get(&DataKey::CurrentLimit(token.clone())).unwrap();
        let limit = current_limit - added_allowed_withdrawal;

        // If the balance is below the limit then set limt to current balance (no withdraws are possible)
        let limit: i128 = if balance < limit {  balance   } else { limit };
                     
        // If limit goes below what the protected percentage is set it to the maxLimit
        let fina_limit = if limit > max_limit { limit } else { max_limit };
        if balance - amount < fina_limit { panic!("exceeds withdraw limit"); };

        env.storage().instance().set(&DataKey::CurrentLimit(token.clone()), &fina_limit);
        env.storage().instance().set(&DataKey::LastUpdate(token.clone()), &env.ledger().timestamp());
    }

    pub fn deposit(
        e: Env,
        from: Address, 
        value: i128,
        token: Address,
        amount: i128,
        to: Option<String>,
        data: Option<BytesN<32>>
    ) {
        if amount < 0 { 
            panic!("Amount less than minimum amount");
        }
       let depositTo = to.unwrap_or(String::from_str(&e, ""));
       let depositData = data.unwrap_or(BytesN::from_array(&e, &[0u8; 32]));

        Self::send_deposit_message(e, from, token, amount, depositTo, depositData, value);
    }

    pub fn deposit_native(
        e: Env,
        from: Address,
        value: i128,
        amount: i128,
        to: Option<String>,
        data: Option<BytesN<32>>
    )  {
        if value < amount { 
            panic!("Amount less than minimum amount");
        }

        let depositTo = to.unwrap_or(String::from_str(&e, ""));
        let depositData = data.unwrap_or(BytesN::from_array(&e, &[0u8; 32]));

        let fee: i128 = value - amount;
        let native_address = read_address_state(&e, DataKey::NativeAddress);
        Self::send_deposit_message(e, from, native_address, amount, depositTo, depositData, fee);
    }

    fn send_deposit_message(
        e: Env,
        from: Address,
        token: Address,
        amount: i128,
        to: String,
        data: BytesN<32>,
        fee: i128
    ) {
        Self::transfer_token_to(e.clone(), from, token, e.current_contract_address(), fee);

        // Messages.Deposit memory xcallMessage = Messages.Deposit(
        //     token.toString(),
        //     msg.sender.toString(),
        //     to,
        //     amount,
        //     data
        // );
        // Messages.DepositRevert memory rollback = Messages.DepositRevert(
        //     token,
        //     amount,
        //     msg.sender
        // );

        let protocols: Map<String, Vec<String>> = Self::get_xcall_manager_client(e).get_protocols();
        // ICallService(xCall).sendCallMessage{value: fee}(
        //     iconAssetManager,
        //     xcallMessage.encodeDeposit(),
        //     rollback.encodeDepositRevert(),
        //     protocols.sources,
        //     protocols.destinations
        // );
    }

    fn get_xcall_manager_client(e: Env) -> Client<'static> {
        return xcall_manager::Client::new(&e, &read_address_state(&e, DataKey::XCallManager));
    }

    pub fn handle_call_message(
        e: Env,
        from: Address,
        data: BytesN<32>,
        protocols: Vec<String>
    )  {
         read_address_state(&e, DataKey::XCall).require_auth();
        if ! Self::get_xcall_manager_client(e).verify_protocols(&protocols) {
          panic!("Protocol Mismatch");
        }

        // string memory method = data.getMethod();
        // if (method.compareTo(Messages.WITHDRAW_TO_NAME)) {
        //     require(from.compareTo(iconAssetManager), "onlyICONAssetManager");
        //     Messages.WithdrawTo memory message = data.decodeWithdrawTo();
        //     withdraw(
        //         message.tokenAddress.parseAddress("Invalid account"),
        //         message.to.parseAddress("Invalid account"),
        //         message.amount
        //     );
        // } else if (method.compareTo(Messages.WITHDRAW_NATIVE_TO_NAME)) {
        //     revert("Withdraw to native is currently not supported");
        // } else if (method.compareTo(Messages.DEPOSIT_REVERT_NAME)) {
        //     require(from.compareTo(xCallNetworkAddress), "onlyCallService");
        //     Messages.DepositRevert memory message = data.decodeDepositRevert();
        //     withdraw(message.tokenAddress, message.to, message.amount);
        // } else {
        //     revert("Unknown message type");
        // }
    }

    pub fn withdraw(e: Env, from: Address, token: Address, to: Address, amount: i128) {
        if amount <= 0 {
            panic!("Amount less than minimum amount");
        }

        Self::verify_withdraw(e.clone(), token.clone(), amount);
        Self::transfer_token_to(e, from, token, to, amount);
    }

    fn transfer_token_to(e: Env, from: Address, token: Address, to: Address, amount: i128){
        let token_client = token::Client::new(&e, &token);
        token_client.transfer_from(&from, &from, &to, &amount);
    }

    pub fn balanceOf(e: Env, token: Address) -> i128 {
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