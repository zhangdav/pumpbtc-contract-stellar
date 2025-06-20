use soroban_sdk::{Address, Env};

use crate::error::PumpBTCStakingError;
use crate::storage::{read_operator, DataKey};

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

pub fn check_operator(e: &Env, address: &Address) -> Result<(), PumpBTCStakingError> {
    let operator = read_operator(e);
    match operator {
        Some(operator) => {
            if &operator != address {
                return Err(PumpBTCStakingError::CallerIsNotOperator);
            }
            address.require_auth();
            Ok(())
        }
        None => Err(PumpBTCStakingError::NoOperatorSet),
    }
}
