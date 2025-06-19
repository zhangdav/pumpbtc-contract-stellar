use crate::storage_types::{DataKey, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{Address, Env};

pub fn extend_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

// ========================= 基础状态管理 =========================

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
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn read_total_staking_cap(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalStakingCap)
        .unwrap_or(0)
}

pub fn write_total_staking_cap(e: &Env, cap: i128) {
    e.storage().instance().set(&DataKey::TotalStakingCap, &cap);
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
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
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
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
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
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
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn read_collected_fee(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::CollectedFee)
        .unwrap_or(0)
}

pub fn write_collected_fee(e: &Env, fee: i128) {
    e.storage().instance().set(&DataKey::CollectedFee, &fee);
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

// ========================= 费用管理 =========================

pub fn read_normal_unstake_fee(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::NormalUnstakeFee)
        .unwrap_or(0)
}

pub fn write_normal_unstake_fee(e: &Env, fee: i128) {
    e.storage().instance().set(&DataKey::NormalUnstakeFee, &fee);
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn read_instant_unstake_fee(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::InstantUnstakeFee)
        .unwrap_or(300) // 默认3%
}

pub fn write_instant_unstake_fee(e: &Env, fee: i128) {
    e.storage()
        .instance()
        .set(&DataKey::InstantUnstakeFee, &fee);
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

// ========================= 地址管理 =========================

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
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
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
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
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
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
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
    // e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn read_pending_unstake_time(e: &Env, user: &Address, slot: u32) -> u64 {
    let key = DataKey::PendingUnstakeTime(user.clone(), slot);
    e.storage().temporary().get(&key).unwrap_or(0)
}

pub fn write_pending_unstake_time(e: &Env, user: &Address, slot: u32, timestamp: u64) {
    let key = DataKey::PendingUnstakeTime(user.clone(), slot);
    e.storage().temporary().set(&key, &timestamp);
    // e.storage().temporary().extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_pending_unstake_amount(e: &Env, user: &Address, slot: u32) -> i128 {
    let key = DataKey::PendingUnstakeAmount(user.clone(), slot);
    e.storage().temporary().get(&key).unwrap_or(0)
}

pub fn write_pending_unstake_amount(e: &Env, user: &Address, slot: u32, amount: i128) {
    let key = DataKey::PendingUnstakeAmount(user.clone(), slot);
    e.storage().temporary().set(&key, &amount);
    // e.storage().temporary().extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}
