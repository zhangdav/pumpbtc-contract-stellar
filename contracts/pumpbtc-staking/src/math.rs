use soroban_sdk::Env;

use crate::error::PumpBTCStakingError;
use crate::storage::read_asset_decimal;

pub fn check_nonnegative_amount(amount: i128) -> Result<(), PumpBTCStakingError> {
    if amount <= 0 {
        return Err(PumpBTCStakingError::NegativeAmountNotAllowed);
    }
    Ok(())
}

pub fn adjust_amount(e: &Env, amount: i128) -> i128 {
    let asset_decimal = read_asset_decimal(e);
    if asset_decimal == 8 {
        amount
    } else {
        let factor = 10i128.pow(safe_sub(asset_decimal as i128, 8).unwrap() as u32);
        safe_mul(amount, factor).unwrap()
    }
}

pub fn safe_mul(a: i128, b: i128) -> Result<i128, PumpBTCStakingError> {
    a.checked_mul(b).ok_or(PumpBTCStakingError::MathOverflow)
}

pub fn safe_add(a: i128, b: i128) -> Result<i128, PumpBTCStakingError> {
    a.checked_add(b).ok_or(PumpBTCStakingError::MathOverflow)
}

pub fn safe_sub(a: i128, b: i128) -> Result<i128, PumpBTCStakingError> {
    a.checked_sub(b).ok_or(PumpBTCStakingError::MathOverflow)
}

pub fn safe_div(a: i128, b: i128) -> Result<i128, PumpBTCStakingError> {
    a.checked_div(b).ok_or(PumpBTCStakingError::MathOverflow)
}
