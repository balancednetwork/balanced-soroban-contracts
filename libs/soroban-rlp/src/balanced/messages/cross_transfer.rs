use crate::decoder;
use crate::encoder;
use soroban_sdk::{contracttype, Bytes, Env, String, Vec};

#[derive(Clone)]
#[contracttype]
pub struct CrossTransfer {
    pub from: String,
    pub to: String,
    pub amount: u128,
    pub data: Bytes,
}

impl CrossTransfer {
    pub fn new(from: String, to: String, amount: u128, data: Bytes) -> Self {
        Self {
            from,
            to,
            amount,
            data,
        }
    }

    pub fn from(&self) -> &String {
        &self.from
    }

    pub fn to(&self) -> &String {
        &self.to
    }

    pub fn encode(&self, e: &Env, method: String) -> Bytes {
        let mut list: Vec<Bytes> = Vec::new(&e);
        list.push_back(encoder::encode_string(&e, method));
        list.push_back(encoder::encode_string(&e, self.from.clone()));
        list.push_back(encoder::encode_string(&e, self.to.clone()));
        list.push_back(encoder::encode_u128(&e, self.amount.clone()));
        list.push_back(encoder::encode(&e, self.data.clone()));

        let encoded = encoder::encode_list(&e, list, false);
        encoded
    }

    pub fn decode(e: &Env, bytes: Bytes) -> CrossTransfer {
        let decoded = decoder::decode_list(&e, bytes);
        if decoded.len() != 5 {
            panic!("InvalidRlpLength");
        }

        let from = decoder::decode_string(e, decoded.get(1).unwrap());
        let to = decoder::decode_string(e, decoded.get(2).unwrap());
        let amount = decoder::decode_u128(e, decoded.get(3).unwrap());
        let data = decoded.get(4).unwrap();
        Self {
            from,
            to,
            amount,
            data,
        }
    }

    pub fn get_method(e: &Env, bytes: Bytes) -> String {
        let decoded = decoder::decode_list(&e, bytes);
        let method = decoder::decode_string(e, decoded.get(0).unwrap());
        method
    }
}
