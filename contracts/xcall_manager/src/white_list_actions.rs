use soroban_sdk::{Bytes, Env, Vec};

use crate::storage_types::DataKey;

#[derive(Clone)]
pub struct WhiteListActions {
    pub key: DataKey,
}

impl WhiteListActions {
    pub fn new(key: DataKey) -> Self {
        Self { key }
    }

    pub fn add(&self, env: &Env, value: Bytes) {
        let mut list = self.get(env);
        list.push_back(value);
        env.storage().instance().set(&self.key, &list);
    }

    pub fn remove(&self, env: &Env, value: Bytes) {
        let mut list = self.get(env);
        if let Some(pos) = list.iter().position(|x| x == value) {
            list.remove(pos as u32);
            env.storage().instance().set(&self.key, &list);
        }
    }

    pub fn contains(&self, env: &Env, value: Bytes) -> bool {
        let list = self.get(env);
        list.contains(&value)
    }

    fn get(&self, env: &Env) -> Vec<Bytes> {
        env.storage().instance().get(&self.key).unwrap_or_else(|| Vec::new(env))
    }
}
