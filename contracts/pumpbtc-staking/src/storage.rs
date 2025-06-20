use soroban_sdk::{contracttype, Address, Env};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const SECONDS_PER_DAY: u64 = 86400;
pub(crate) const UTC_OFFSET: u64 = 8 * 3600;

pub(crate) const MAX_DATE_SLOT: u32 = 10;

// Instance storage keys - for contract configuration and management data
#[derive(Clone)]
#[contracttype]
pub enum InstanceDataKey {
    Admin,
    PendingAdmin,
    Paused,
    Operator,
    PumpTokenAddress,
    AssetTokenAddress,
    AssetDecimal,
    NormalUnstakeFee,
    InstantUnstakeFee,
    OnlyAllowStake,
}

// Persistent storage keys - for long-term global state
#[derive(Clone)]
#[contracttype]
pub enum PersistentDataKey {
    TotalStakingAmount,
    TotalStakingCap,
    TotalRequestedAmount,
    TotalClaimableAmount,
    PendingStakeAmount,
    CollectedFee,
}

// Temporary storage keys - for time-limited user data
#[derive(Clone)]
#[contracttype]
pub enum TemporaryDataKey {
    PendingUnstakeTime(Address, u32),
    PendingUnstakeAmount(Address, u32),
}

pub fn read_pump_token_address(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&InstanceDataKey::PumpTokenAddress)
        .unwrap()
}

pub fn write_pump_token_address(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&InstanceDataKey::PumpTokenAddress, address);
}

pub fn read_asset_token_address(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&InstanceDataKey::AssetTokenAddress)
        .unwrap()
}

pub fn write_asset_token_address(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&InstanceDataKey::AssetTokenAddress, address);
}

pub fn read_asset_decimal(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&InstanceDataKey::AssetDecimal)
        .unwrap_or(8)
}

pub fn write_asset_decimal(e: &Env, decimal: u32) {
    e.storage()
        .instance()
        .set(&InstanceDataKey::AssetDecimal, &decimal);
}

pub fn read_operator(e: &Env) -> Option<Address> {
    e.storage().instance().get(&InstanceDataKey::Operator)
}

pub fn write_operator(e: &Env, operator: &Address) {
    e.storage()
        .instance()
        .set(&InstanceDataKey::Operator, operator);
}

pub fn read_normal_unstake_fee(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&InstanceDataKey::NormalUnstakeFee)
        .unwrap_or(0)
}

pub fn write_normal_unstake_fee(e: &Env, fee: i128) {
    e.storage()
        .instance()
        .set(&InstanceDataKey::NormalUnstakeFee, &fee);
}

pub fn read_instant_unstake_fee(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&InstanceDataKey::InstantUnstakeFee)
        .unwrap_or(300) // default 3%
}

pub fn write_instant_unstake_fee(e: &Env, fee: i128) {
    e.storage()
        .instance()
        .set(&InstanceDataKey::InstantUnstakeFee, &fee);
}

pub fn read_only_allow_stake(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&InstanceDataKey::OnlyAllowStake)
        .unwrap_or(true)
}

pub fn write_only_allow_stake(e: &Env, only_allow_stake: bool) {
    e.storage()
        .instance()
        .set(&InstanceDataKey::OnlyAllowStake, &only_allow_stake);
}

pub fn read_paused(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&InstanceDataKey::Paused)
        .unwrap_or(false)
}

pub fn write_paused(e: &Env, paused: bool) {
    e.storage()
        .instance()
        .set(&InstanceDataKey::Paused, &paused);
}

pub fn read_total_staking_amount(e: &Env) -> i128 {
    e.storage()
        .persistent()
        .get(&PersistentDataKey::TotalStakingAmount)
        .unwrap_or(0)
}

pub fn write_total_staking_amount(e: &Env, amount: i128) {
    e.storage()
        .persistent()
        .set(&PersistentDataKey::TotalStakingAmount, &amount);
}

pub fn read_total_staking_cap(e: &Env) -> i128 {
    e.storage()
        .persistent()
        .get(&PersistentDataKey::TotalStakingCap)
        .unwrap_or(0)
}

pub fn write_total_staking_cap(e: &Env, cap: i128) {
    e.storage()
        .persistent()
        .set(&PersistentDataKey::TotalStakingCap, &cap);
}

pub fn read_total_requested_amount(e: &Env) -> i128 {
    e.storage()
        .persistent()
        .get(&PersistentDataKey::TotalRequestedAmount)
        .unwrap_or(0)
}

pub fn write_total_requested_amount(e: &Env, amount: i128) {
    e.storage()
        .persistent()
        .set(&PersistentDataKey::TotalRequestedAmount, &amount);
}

pub fn read_total_claimable_amount(e: &Env) -> i128 {
    e.storage()
        .persistent()
        .get(&PersistentDataKey::TotalClaimableAmount)
        .unwrap_or(0)
}

pub fn write_total_claimable_amount(e: &Env, amount: i128) {
    e.storage()
        .persistent()
        .set(&PersistentDataKey::TotalClaimableAmount, &amount);
}

pub fn read_pending_stake_amount(e: &Env) -> i128 {
    e.storage()
        .persistent()
        .get(&PersistentDataKey::PendingStakeAmount)
        .unwrap_or(0)
}

pub fn write_pending_stake_amount(e: &Env, amount: i128) {
    e.storage()
        .persistent()
        .set(&PersistentDataKey::PendingStakeAmount, &amount);
}

pub fn read_collected_fee(e: &Env) -> i128 {
    e.storage()
        .persistent()
        .get(&PersistentDataKey::CollectedFee)
        .unwrap_or(0)
}

pub fn write_collected_fee(e: &Env, fee: i128) {
    e.storage()
        .persistent()
        .set(&PersistentDataKey::CollectedFee, &fee);
}

pub fn read_pending_unstake_time(e: &Env, user: &Address, slot: u32) -> u64 {
    let key = TemporaryDataKey::PendingUnstakeTime(user.clone(), slot);
    e.storage().temporary().get(&key).unwrap_or(0)
}

pub fn write_pending_unstake_time(e: &Env, user: &Address, slot: u32, timestamp: u64) {
    let key = TemporaryDataKey::PendingUnstakeTime(user.clone(), slot);
    e.storage().temporary().set(&key, &timestamp);
}

pub fn read_pending_unstake_amount(e: &Env, user: &Address, slot: u32) -> i128 {
    let key = TemporaryDataKey::PendingUnstakeAmount(user.clone(), slot);
    e.storage().temporary().get(&key).unwrap_or(0)
}

pub fn write_pending_unstake_amount(e: &Env, user: &Address, slot: u32, amount: i128) {
    let key = TemporaryDataKey::PendingUnstakeAmount(user.clone(), slot);
    e.storage().temporary().set(&key, &amount);
}
