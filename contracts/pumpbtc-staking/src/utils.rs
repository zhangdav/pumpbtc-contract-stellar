use crate::math::{safe_add, safe_div};
use soroban_sdk::Env;

use crate::error::PumpBTCStakingError;
use crate::storage::{
    read_only_allow_stake, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, MAX_DATE_SLOT,
    SECONDS_PER_DAY, UTC_OFFSET,
};

pub fn check_unstake_allowed(e: &Env) -> Result<(), PumpBTCStakingError> {
    if read_only_allow_stake(e) {
        return Err(PumpBTCStakingError::OnlyAllowStakeAtFirst);
    }
    Ok(())
}

pub fn get_date_slot(timestamp: u64) -> u32 {
    (safe_div(
        safe_add(timestamp as i128, UTC_OFFSET as i128).unwrap(),
        SECONDS_PER_DAY as i128,
    )
    .unwrap()
        % (MAX_DATE_SLOT as i128)) as u32
}

pub fn extend_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}
