use soroban_sdk::{Address, Env};

use crate::storage_types::DataKey;

pub fn read_minter(e: &Env) -> Address {
    let key = DataKey::Minter;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_minter(e: &Env, id: &Address) {
    let key = DataKey::Minter;
    e.storage().instance().set(&key, id);
}
