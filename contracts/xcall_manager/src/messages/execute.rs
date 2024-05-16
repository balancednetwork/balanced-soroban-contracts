use soroban_sdk::{contracttype, Env, String, Bytes, Vec, Address};
use soroban_rlp::{encoder, decoder};
use crate::errors::ContractError;

#[derive(Clone)]
#[contracttype]
pub struct Execute {
    pub contract_address: Address,    
    pub data: Bytes
}  

impl Execute{
    pub fn new(contract_address: Address, data: Bytes) -> Self {
        Self {
            contract_address,
            data
        }
    }

    pub fn contract_address(&self) -> &Address {
        &self.contract_address
    }

    pub fn data(&self) -> &Bytes {
        &self.data
    }

    pub fn encode(&self, e: &Env, method: String) -> Bytes {
        let mut list: Vec<Bytes> = Vec::new(&e);
        list.push_back(encoder::encode_string(&e, method));
        list.push_back(encoder::encode_string(&e, self.contract_address.clone().to_string() ));
        list.push_back(encoder::encode(&e, self.data.clone()));

        let encoded = encoder::encode_list(&e, list, false);
        encoded
    }

    pub fn decode(e: &Env, bytes: Bytes) -> Result<Execute, ContractError> {
        let decoded = decoder::decode_list(&e, bytes);
        if decoded.len() != 6 {
            return Err(ContractError::InvalidRlpLength);
        }

        let contract_address = Address::from_string(&decoder::decode_string(e, decoded.get(1).unwrap()));
        let data = decoded.get(2).unwrap();

        Ok(Self {
            contract_address,
            data
        })
    }
}