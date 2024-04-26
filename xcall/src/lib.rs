#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec, Bytes};

#[contract]
pub struct Xcall;

#[contractimpl]
impl Xcall {
    
    pub fn send_call_message(env: Env, from: Address, amount: i128, icon_asset_manager: String, data: Bytes, rollback: Bytes, sources: Vec<String>, destinations: Vec<String>){
        
    }

    pub fn get_network_address(env: Env, address: String)-> String {
        address
    }

}