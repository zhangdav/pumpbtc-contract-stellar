use soroban_sdk::{contracttype, Address, Env};

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
    PendingUnstakeTime(Address, u32),
    PendingUnstakeAmount(Address, u32),
}

pub fn read_total_staking_amount(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalStakingAmount)
        .unwrap_or(0)
}

pub fn write_total_staking_amount(e: &Env, amount: i128) {
    e.storage()
        .instance()
        .set(&DataKey::TotalStakingAmount, &amount);
}

pub fn read_total_staking_cap(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalStakingCap)
        .unwrap_or(0)
}

pub fn write_total_staking_cap(e: &Env, cap: i128) {
    e.storage().instance().set(&DataKey::TotalStakingCap, &cap);
}

pub fn read_total_requested_amount(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalRequestedAmount)
        .unwrap_or(0)
}

pub fn write_total_requested_amount(e: &Env, amount: i128) {
    e.storage()
        .instance()
        .set(&DataKey::TotalRequestedAmount, &amount);
}

pub fn read_total_claimable_amount(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalClaimableAmount)
        .unwrap_or(0)
}

pub fn write_total_claimable_amount(e: &Env, amount: i128) {
    e.storage()
        .instance()
        .set(&DataKey::TotalClaimableAmount, &amount);
}

pub fn read_pending_stake_amount(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::PendingStakeAmount)
        .unwrap_or(0)
}

pub fn write_pending_stake_amount(e: &Env, amount: i128) {
    e.storage()
        .instance()
        .set(&DataKey::PendingStakeAmount, &amount);
}

pub fn read_collected_fee(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::CollectedFee)
        .unwrap_or(0)
}

pub fn write_collected_fee(e: &Env, fee: i128) {
    e.storage().instance().set(&DataKey::CollectedFee, &fee);
}

pub fn read_normal_unstake_fee(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::NormalUnstakeFee)
        .unwrap_or(0)
}

pub fn write_normal_unstake_fee(e: &Env, fee: i128) {
    e.storage().instance().set(&DataKey::NormalUnstakeFee, &fee);
}

pub fn read_instant_unstake_fee(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::InstantUnstakeFee)
        .unwrap_or(300) // default 3%
}

pub fn write_instant_unstake_fee(e: &Env, fee: i128) {
    e.storage()
        .instance()
        .set(&DataKey::InstantUnstakeFee, &fee);
}

pub fn read_pump_token_address(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::PumpTokenAddress)
        .unwrap()
}

pub fn write_pump_token_address(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::PumpTokenAddress, address);
}

pub fn read_asset_token_address(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::AssetTokenAddress)
        .unwrap()
}

pub fn write_asset_token_address(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::AssetTokenAddress, address);
}

pub fn read_asset_decimal(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&DataKey::AssetDecimal)
        .unwrap_or(8)
}

pub fn write_asset_decimal(e: &Env, decimal: u32) {
    e.storage().instance().set(&DataKey::AssetDecimal, &decimal);
}

pub fn read_operator(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::Operator)
}

pub fn write_operator(e: &Env, operator: &Address) {
    e.storage().instance().set(&DataKey::Operator, operator);
}

pub fn read_only_allow_stake(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::OnlyAllowStake)
        .unwrap_or(true)
}

pub fn write_only_allow_stake(e: &Env, only_allow_stake: bool) {
    e.storage()
        .instance()
        .set(&DataKey::OnlyAllowStake, &only_allow_stake);
}

pub fn read_pending_unstake_time(e: &Env, user: &Address, slot: u32) -> u64 {
    let key = DataKey::PendingUnstakeTime(user.clone(), slot);
    e.storage().temporary().get(&key).unwrap_or(0)
}

pub fn write_pending_unstake_time(e: &Env, user: &Address, slot: u32, timestamp: u64) {
    let key = DataKey::PendingUnstakeTime(user.clone(), slot);
    e.storage().temporary().set(&key, &timestamp);
}

pub fn read_pending_unstake_amount(e: &Env, user: &Address, slot: u32) -> i128 {
    let key = DataKey::PendingUnstakeAmount(user.clone(), slot);
    e.storage().temporary().get(&key).unwrap_or(0)
}

pub fn write_pending_unstake_amount(e: &Env, user: &Address, slot: u32, amount: i128) {
    let key = DataKey::PendingUnstakeAmount(user.clone(), slot);
    e.storage().temporary().set(&key, &amount);
}
