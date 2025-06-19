use soroban_sdk::{contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewTotalStakingCapEvent {
    pub old_total_staking_cap: i128,
    pub new_total_staking_cap: i128,
}

pub(crate) fn set_stake_asset_cap(
    e: &Env,
    old_total_staking_cap: i128,
    new_total_staking_cap: i128,
) {
    let event: NewTotalStakingCapEvent = NewTotalStakingCapEvent {
        old_total_staking_cap,
        new_total_staking_cap,
    };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("set_cap")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewNormalUnstakeFeeEvent {
    pub old_normal_unstake_fee: i128,
    pub new_normal_unstake_fee: i128,
}

pub(crate) fn set_normal_unstake_fee(
    e: &Env,
    old_normal_unstake_fee: i128,
    new_normal_unstake_fee: i128,
) {
    let event: NewNormalUnstakeFeeEvent = NewNormalUnstakeFeeEvent {
        old_normal_unstake_fee,
        new_normal_unstake_fee,
    };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("set_nfee")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetInstantUnstakeFeeEvent {
    pub old_instant_unstake_fee: i128,
    pub new_instant_unstake_fee: i128,
}

pub(crate) fn set_instant_unstake_fee(
    e: &Env,
    old_instant_unstake_fee: i128,
    new_instant_unstake_fee: i128,
) {
    let event: SetInstantUnstakeFeeEvent = SetInstantUnstakeFeeEvent {
        old_instant_unstake_fee,
        new_instant_unstake_fee,
    };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("set_ifee")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetOperatorEvent {
    pub old_operator: Option<Address>,
    pub new_operator: Address,
}

pub(crate) fn set_operator(e: &Env, old_operator: Option<Address>, new_operator: Address) {
    let event: SetOperatorEvent = SetOperatorEvent {
        old_operator: old_operator,
        new_operator: new_operator,
    };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("set_op")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetOnlyAllowStakeEvent {
    pub only_allow_stake: bool,
}

pub(crate) fn set_only_allow_stake(e: &Env, only_allow_stake: bool) {
    let event: SetOnlyAllowStakeEvent = SetOnlyAllowStakeEvent { only_allow_stake };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("set_allow")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectFeeEvent {
    pub admin: Address,
    pub fee_amount: i128,
}

pub(crate) fn collect_fee(e: &Env, admin: Address, fee_amount: i128) {
    let event: CollectFeeEvent = CollectFeeEvent { admin, fee_amount };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("collect")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawEvent {
    pub operator: Address,
    pub amount: i128,
}

pub(crate) fn withdraw(e: &Env, operator: Address, amount: i128) {
    let event: WithdrawEvent = WithdrawEvent { operator, amount };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("withdraw")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositEvent {
    pub operator: Address,
    pub pumpbtc_staking: Address,
    pub amount: i128,
}

pub(crate) fn deposit(e: &Env, operator: Address, pumpbtc_staking: Address, amount: i128) {
    let event: DepositEvent = DepositEvent {
        operator,
        pumpbtc_staking,
        amount,
    };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("deposit")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakeEvent {
    pub user: Address,
    pub amount: i128,
}

pub(crate) fn stake(e: &Env, user: Address, amount: i128) {
    let event: StakeEvent = StakeEvent {
        user,
        amount,
    };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("stake")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnstakeRequestEvent {
    pub user: Address,
    pub amount: i128,
    pub slot: u32,
}

pub(crate) fn unstake_request(e: &Env, user: Address, amount: i128, slot: u32) {
    let event: UnstakeRequestEvent = UnstakeRequestEvent {
        user,
        amount,
        slot,
    };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("unstake")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimSlotEvent {
    pub user: Address,
    pub amount: i128,
    pub slot: u32,
}

pub(crate) fn claim_slot(e: &Env, user: Address, amount: i128, slot: u32) {
    let event: ClaimSlotEvent = ClaimSlotEvent {
        user,
        amount,
        slot,
    };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("claim")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnstakeInstantEvent {
    pub user: Address,
    pub amount: i128,
}

pub(crate) fn unstake_instant(e: &Env, user: Address, amount: i128) {
    let event: UnstakeInstantEvent = UnstakeInstantEvent {
        user,
        amount,
    };
    e.events()
        .publish(("PumpBTCStaking", symbol_short!("unstake_i")), event);
}