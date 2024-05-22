use soroban_sdk::{contracttype, Env, String, Address, Bytes, Vec};
use crate::encoder;
use crate::decoder;

#[derive(Clone)]
#[contracttype]
pub struct DepositRevert {
    pub token_address: Address,
    pub to: Address,
    pub amount: u128
}

impl DepositRevert{
    pub fn new(token_address: Address, to: Address, amount: u128) -> Self {
        Self {
            token_address,
            to,
            amount,
        }
    }

    pub fn token_address(&self) -> &Address {
        &self.token_address
    }

    pub fn to(&self) -> &Address {
        &self.to
    }

    pub fn amount(&self) -> &u128 {
        &self.amount
    }

    pub fn encode(&self, e: &Env, method: String) -> Bytes {
        let mut list: Vec<Bytes> = Vec::new(&e);
        list.push_back(encoder::encode_string(&e, method));
        list.push_back(encoder::encode_string(&e, self.token_address.to_string().clone()));
        list.push_back(encoder::encode_string(&e, self.to.to_string().clone()));
        list.push_back(encoder::encode_u128(&e, self.amount.clone()));

        let encoded = encoder::encode_list(&e, list, false);
        encoded
    }

    pub fn decode(e: &Env, bytes: Bytes) -> DepositRevert {
        let decoded = decoder::decode_list(&e, bytes);
        if decoded.len() != 4 {
             panic!("InvalidRlpLength");
        }

        let token_address = Address::from_string(&decoder::decode_string(e, decoded.get(1).unwrap()));
        let to = Address::from_string(&decoder::decode_string(e, decoded.get(2).unwrap()));
        let amount = decoder::decode_u128(e, decoded.get(3).unwrap());

        Self {
            token_address,
            to,
            amount
        }
    }
}