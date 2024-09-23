use soroban_sdk::{contracttype, Env, String, Bytes, Vec};
use crate::encoder;
use crate::decoder;

#[derive(Clone)]
#[contracttype]
pub struct WithdrawTo {
    pub token_address: String,
    pub to: String,
    pub amount: u128
}

impl WithdrawTo{
    pub fn new(token_address: String, to: String, amount: u128) -> Self {
        Self {
            token_address,
            to,
            amount
        }
    }

    pub fn token_address(&self) -> &String {
        &self.token_address
    }

    pub fn to(&self) -> &String {
        &self.to
    }

    pub fn amount(&self) -> &u128 {
        &self.amount
    }

    pub fn encode(&self, e: &Env, method: String) -> Bytes {
        let mut list: Vec<Bytes> = Vec::new(&e);
        list.push_back(encoder::encode_string(&e, method));
        list.push_back(encoder::encode_string(&e, self.token_address.clone()));
        list.push_back(encoder::encode_string(&e, self.to.clone()));
        list.push_back(encoder::encode_u128(&e, self.amount.clone()));

        let encoded = encoder::encode_list(&e, list, false);
        encoded
    }

    pub fn decode(e: &Env, bytes: Bytes) -> WithdrawTo {
        let decoded = decoder::decode_list(&e, bytes);
        if decoded.len() != 4 {
            panic!("InvalidRlpLength");
        }

        let token_address = decoder::decode_string(e, decoded.get(1).unwrap());
        let to = decoder::decode_string(e, decoded.get(2).unwrap());
        let amount = decoder::decode_u128(e, decoded.get(3).unwrap());

        Self {
            token_address,
            to,
            amount
        }
    }
}