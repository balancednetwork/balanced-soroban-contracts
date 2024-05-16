use soroban_sdk::{token::TokenInterface, Address, Bytes, Env, String, Vec};

mod xcall {
    soroban_sdk::contractimport!(file = "xcall.wasm");
}

use crate::{
    admin::read_administrator, config::{get_config, ConfigData}, contract::BalancedDollar, messages::{cross_transfer::CrossTransfer, cross_transfer_revert::CrossTransferRevert}, storage_types::DataKey, xcall_manager_interface::XcallManagerClient
};

use crate::errors::ContractError;

use xcall::{AnyMessage, CallMessageWithRollback, Client, Envelope};

const CROSS_TRANSFER: &str = "xCrossTransfer";
const CROSS_TRANSFER_REVERT: &str = "xCrossTransferRevert";

impl BalancedDollar {

    pub fn configure(env:Env, xcall: Address, 
        xcall_manager: Address, nid: String, icon_bn_usd: String, xcall_network_address: String){
            let admin = read_administrator(&env.clone());
            admin.require_auth();

            let config: ConfigData = ConfigData{ xcall: xcall, xcall_manager: xcall_manager, icon_bn_usd: icon_bn_usd, nid: nid, xcall_network_address: xcall_network_address };
            env.storage().instance().set(&DataKey::Config, &config);
    }

    pub fn cross_transfer(
        e: Env,
        from: Address,
        amount: u128,
        to: String,
        value: u128
    ) {
        from.require_auth();
        Self::_cross_transfer(e.clone(), from, amount, to, value, Bytes::new(&e)).unwrap();
    }

    pub fn cross_transfer_data(
        e: Env,
        from: Address,
        amount: u128,
        to: String,
        value: u128,
        data: Bytes
    ) {
        from.require_auth();
        Self::_cross_transfer(e, from, amount, to, value, data).unwrap();
    }

     fn _cross_transfer(
        e: Env,
        from: Address,
        amount: u128,
        to: String,
        value: u128,
        data: Bytes
    )  -> Result<(), ContractError> {
        if value == 0 {
            panic!("Amount less than minimum amount");
        }
       Self::burn(e.clone(), from.clone(), u128::try_into(amount).unwrap());
        let xcall_message = CrossTransfer::new(
            from.clone().to_string(),
            to,
            value,
            data
        );

        let rollback = CrossTransferRevert::new(
            from.clone(),
            value
        );

        let rollback_bytes = rollback.encode(&e, String::from_str(&e.clone(), CROSS_TRANSFER_REVERT));
        let message_bytes = xcall_message.encode(&e, String::from_str(&e.clone(), CROSS_TRANSFER));
        let (sources, destinations) = Self::xcall_manager(e.clone()).get_protocols();

        let message = AnyMessage::CallMessageWithRollback(CallMessageWithRollback { data: message_bytes, rollback: rollback_bytes });
        let envelope: &Envelope = &Envelope {
            message,
            sources,
            destinations
        };
        let icon_bn_usd = &get_config(&e).icon_bn_usd;
        let current_address = e.clone().current_contract_address();
        Self::xcall_client(e).send_call(&from, &current_address, envelope, icon_bn_usd );
        Ok(())

    }

    
    pub fn handle_call_message(
        e: Env,
        from: String,
        data: Bytes,
        protocols: Vec<String>
    ) {
        let xcall = get_config(&e).xcall;
        xcall.require_auth();
        if !Self::xcall_manager(e.clone()).verify_protocols(&protocols) {
            panic!("Protocol Mismatch");
        };

        let method = CrossTransfer::get_method(&e, data.clone()).unwrap();
        let icon_bn_usd = get_config(&e).icon_bn_usd;
        if method == String::from_str(&e, &CROSS_TRANSFER){
            if from!=icon_bn_usd {
                panic!("onlyICONBnUSD");
            }
            let message = CrossTransfer::decode(&e.clone(), data).unwrap();
            Self::mint(e.clone(), Address::from_string( &message.to), u128::try_into(message.amount).unwrap());
        } else if method == String::from_str(&e, &CROSS_TRANSFER_REVERT){
            if from!=xcall.to_string() {
                panic!("onlyCallService");
            }
            let message = CrossTransferRevert::decode(&e.clone(), data).unwrap();
            Self::mint(e.clone(), message.to, u128::try_into(message.amount).unwrap());
        }else{
            panic!("Unknown message type")
        }
    }

    fn xcall_client(e: Env) -> Client<'static> {
        return xcall::Client::new(&e, &get_config(&e).xcall);
    }

    fn xcall_manager(e: Env) -> XcallManagerClient<'static> {
        let client = XcallManagerClient::new(&e, &get_config(&e).xcall_manager);
        return client;
     }

}