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
        let factor = 10i128.pow(asset_decimal - 8);
        amount * factor
    }
}
