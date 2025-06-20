use soroban_sdk::{Address, Env};

use crate::storage_types::DataKey;

pub fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().instance().has(&key)
}

pub fn read_administrator(e: &Env) -> Address {
    let key = DataKey::Admin;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    let key = DataKey::Admin;
    e.storage().instance().set(&key, id);
}

pub fn read_pending_administrator(e: &Env) -> Option<Address> {
    let key = DataKey::PendingAdmin;
    e.storage().instance().get(&key)
}

pub fn write_pending_administrator(e: &Env, id: &Address) {
    let key = DataKey::PendingAdmin;
    e.storage().instance().set(&key, id);
}

pub fn remove_pending_administrator(e: &Env) {
    let key = DataKey::PendingAdmin;
    e.storage().instance().remove(&key);
}
