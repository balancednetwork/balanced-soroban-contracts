use soroban_sdk::{contracttype, Env, String, Address, Bytes, Vec};
use crate::encoder;
use crate::decoder;

#[derive(Clone)]
#[contracttype]
pub struct CrossTransferRevert {
    pub to: Address,
    pub amount: u128
}

impl CrossTransferRevert{
    pub fn new(to: Address, amount: u128) -> Self {
        Self {
            to,
            amount,
        }
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
        list.push_back(encoder::encode_string(&e, self.to.clone().to_string()));
        list.push_back(encoder::encode_u128(&e, self.amount.clone()));

        let encoded = encoder::encode_list(&e, list, false);
        encoded
    }

    pub fn decode(e: &Env, bytes: Bytes) -> CrossTransferRevert{
        let decoded = decoder::decode_list(&e, bytes);
        if decoded.len() != 3 {
            panic!("InvalidRlpLength");
        }

        let to = Address::from_string(&decoder::decode_string(e, decoded.get(1).unwrap()));
        let amount = decoder::decode_u128(e, decoded.get(2).unwrap());

        Self {
            to,
            amount
        }
    }
}