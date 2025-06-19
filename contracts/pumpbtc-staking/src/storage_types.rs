use soroban_sdk::{contracttype, Address};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const SECONDS_PER_DAY: u64 = 86400;
pub(crate) const UTC_OFFSET: u64 = 8 * 3600;

pub(crate) const MAX_DATE_SLOT: u32 = 10;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Operator,
    PumpTokenAddress,
    AssetTokenAddress,
    AssetDecimal,
    TotalStakingAmount,
    TotalStakingCap,
    TotalRequestedAmount,
    TotalClaimableAmount,
    PendingStakeAmount,
    CollectedFee,
    NormalUnstakeFee,
    InstantUnstakeFee,
    OnlyAllowStake,
    PendingUnstakeTime(Address, u32),   // (user, slot)
    PendingUnstakeAmount(Address, u32), // (user, slot)
}

#[derive(Clone)]
#[contracttype]
pub struct StakingState {
    pub total_staking_amount: i128,
    pub total_staking_cap: i128,
    pub total_requested_amount: i128,
    pub total_claimable_amount: i128,
    pub pending_stake_amount: i128,
    pub collected_fee: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct UnstakeRequest {
    pub amount: i128,
    pub timestamp: u64,
}
