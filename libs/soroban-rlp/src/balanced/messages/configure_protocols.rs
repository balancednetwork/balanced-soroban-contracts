use crate::decoder;
use crate::encoder;
use soroban_sdk::{contracttype, Bytes, Env, String, Vec};

#[derive(Clone)]
#[contracttype]
pub struct ConfigureProtocols {
    pub sources: Vec<String>,
    pub destinations: Vec<String>,
}

impl ConfigureProtocols {
    pub fn new(sources: Vec<String>, destinations: Vec<String>) -> Self {
        Self {
            sources,
            destinations,
        }
    }

    pub fn sources(&self) -> &Vec<String> {
        &self.sources
    }

    pub fn destinations(&self) -> &Vec<String> {
        &self.destinations
    }

    pub fn encode(&self, e: &Env, method: String) -> Bytes {
        let mut list: Vec<Bytes> = Vec::new(&e);
        list.push_back(encoder::encode_string(&e, method));
        list.push_back(encoder::encode_strings(&e, self.sources.clone()));
        list.push_back(encoder::encode_strings(&e, self.destinations.clone()));

        let encoded = encoder::encode_list(&e, list, false);
        encoded
    }

    pub fn decode(e: &Env, bytes: Bytes) -> ConfigureProtocols {
        let decoded = decoder::decode_list(&e, bytes);
        if decoded.len() != 3 {
            panic!("InvalidRlpLength");
        }

        let sources = decoder::decode_strings(e, decoded.get(1).unwrap());
        let destinations = decoder::decode_strings(e, decoded.get(2).unwrap());
        Self {
            sources,
            destinations,
        }
    }

    pub fn get_method(e: &Env, bytes: Bytes) -> String {
        let decoded = decoder::decode_list(&e, bytes);
        let method = decoder::decode_string(e, decoded.get(0).unwrap());
        method
    }
}
