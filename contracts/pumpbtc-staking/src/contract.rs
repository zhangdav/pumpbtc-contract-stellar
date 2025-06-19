use crate::admin::{has_administrator, read_administrator, write_administrator};
use crate::error::PumpBTCStakingError;
use crate::event;
use crate::storage::*;
use crate::storage_types::{MAX_DATE_SLOT, SECONDS_PER_DAY, UTC_OFFSET};
use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct PumpBTCStaking;

// ========================= Utils Functions =========================

fn check_nonnegative_amount(amount: i128) -> Result<(), PumpBTCStakingError> {
    if amount < 0 {
        return Err(PumpBTCStakingError::NegativeAmountNotAllowed);
    }
    Ok(())
}

fn get_date_slot(timestamp: u64) -> u32 {
    (((timestamp + UTC_OFFSET) / SECONDS_PER_DAY) % (MAX_DATE_SLOT as u64)) as u32
}

fn adjust_amount(e: &Env, amount: i128) -> i128 {
    let asset_decimal = read_asset_decimal(e);
    if asset_decimal == 8 {
        amount
    } else {
        let factor = 10i128.pow(asset_decimal - 8);
        amount * factor
    }
}

fn check_operator(e: &Env, address: &Address) -> Result<(), PumpBTCStakingError> {
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

fn check_unstake_allowed(e: &Env) -> Result<(), PumpBTCStakingError> {
    if read_only_allow_stake(e) {
        return Err(PumpBTCStakingError::OnlyAllowStakeAtFirst);
    }
    Ok(())
}

#[contractimpl]
impl PumpBTCStaking {
    fn initialize(
        e: Env,
        admin: Address,
        pump_token_address: Address,
        asset_token_address: Address,
    ) -> Result<(), PumpBTCStakingError> {
        if !has_administrator(&e) {
            write_administrator(&e, &admin);

            write_pump_token_address(&e, &pump_token_address);
            write_asset_token_address(&e, &asset_token_address);

            let asset_client = token::Client::new(&e, &asset_token_address);
            let asset_decimal = asset_client.decimals();
            write_asset_decimal(&e, asset_decimal);

            write_normal_unstake_fee(&e, 0);
            write_instant_unstake_fee(&e, 300);
            write_only_allow_stake(&e, true);

            extend_instance_ttl(&e);
            Ok(())
        } else {
            return Err(PumpBTCStakingError::AlreadyInitialized);
        }
    }

    // ========================= Owner Functions =========================

    fn set_stake_asset_cap(e: Env, new_total_staking_cap: i128) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        let admin = read_administrator(&e);
        admin.require_auth();

        check_nonnegative_amount(new_total_staking_cap)?;
        let total_staking_amount = read_total_staking_amount(&e);

        if new_total_staking_cap >= total_staking_amount {
            let old_total_staking_cap = read_total_staking_cap(&e);
            write_total_staking_cap(&e, new_total_staking_cap);

            event::set_stake_asset_cap(&e, old_total_staking_cap, new_total_staking_cap);
            Ok(())
        } else {
            return Err(PumpBTCStakingError::StakingCapTooSmall);
        }
    }

    fn set_normal_unstake_fee(
        e: Env,
        new_normal_unstake_fee: i128,
    ) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        let admin = read_administrator(&e);
        admin.require_auth();

        if new_normal_unstake_fee < 10000 {
            let old_normal_unstake_fee = read_normal_unstake_fee(&e);

            write_normal_unstake_fee(&e, new_normal_unstake_fee);
            event::set_normal_unstake_fee(&e, old_normal_unstake_fee, new_normal_unstake_fee);
            Ok(())
        } else {
            return Err(PumpBTCStakingError::FeeShouldBeBetween0And10000);
        }
    }

    fn set_instant_unstake_fee(
        e: Env,
        new_instant_unstake_fee: i128,
    ) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        let admin = read_administrator(&e);
        admin.require_auth();

        if new_instant_unstake_fee < 10000 {
            let old_instant_unstake_fee = read_instant_unstake_fee(&e);

            write_instant_unstake_fee(&e, new_instant_unstake_fee);
            event::set_instant_unstake_fee(&e, old_instant_unstake_fee, new_instant_unstake_fee);
            Ok(())
        } else {
            return Err(PumpBTCStakingError::FeeShouldBeBetween0And10000);
        }
    }

    fn set_operator(e: Env, new_operator: Address) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        let admin = read_administrator(&e);
        admin.require_auth();

        let old_operator = read_operator(&e);

        write_operator(&e, &new_operator);
        event::set_operator(&e, old_operator, new_operator);

        Ok(())
    }

    fn set_only_allow_stake(e: Env, only_allow_stake: bool) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        let admin = read_administrator(&e);
        admin.require_auth();

        write_only_allow_stake(&e, only_allow_stake);
        event::set_only_allow_stake(&e, only_allow_stake);

        Ok(())
    }

    fn collect_fee(e: Env) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        let admin = read_administrator(&e);
        admin.require_auth();

        let fee_amount = read_collected_fee(&e);
        if fee_amount > 0 {
            write_collected_fee(&e, 0);

            let asset_token = read_asset_token_address(&e);

            let asset_client = token::Client::new(&e, &asset_token);
            asset_client.transfer(
                &e.current_contract_address(),
                &admin,
                &adjust_amount(&e, fee_amount),
            );

            event::collect_fee(&e, admin, fee_amount);
            Ok(())
        } else {
            return Err(PumpBTCStakingError::NoFeeToCollect);
        }
    }

    // ========================= Operator Functions =========================

    fn withdraw(e: Env) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        let operator = read_operator(&e);
        if operator.is_none() {
            return Err(PumpBTCStakingError::NoOperatorSet);
        }

        let operator = operator.unwrap();
        check_operator(&e, &operator)?;

        let old_pending_amount = read_pending_stake_amount(&e);
        if old_pending_amount > 0 {
            write_pending_stake_amount(&e, 0);

            let asset_token = read_asset_token_address(&e);
            let asset_client = token::Client::new(&e, &asset_token);
            asset_client.transfer(
                &e.current_contract_address(),
                &operator,
                &adjust_amount(&e, old_pending_amount),
            );

            event::withdraw(&e, operator, old_pending_amount);
            Ok(())
        } else {
            return Err(PumpBTCStakingError::NoPendingStakeAmount);
        }
    }

    fn deposit(e: Env, amount: i128) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        let operator = read_operator(&e);
        if operator.is_none() {
            return Err(PumpBTCStakingError::NoOperatorSet);
        }
        let operator = operator.unwrap();
        check_operator(&e, &operator)?;

        check_nonnegative_amount(amount)?;

        let total_claimable_amount = read_total_claimable_amount(&e);
        write_total_claimable_amount(&e, total_claimable_amount + amount);

        let asset_token = read_asset_token_address(&e);
        let asset_client = token::Client::new(&e, &asset_token);
        asset_client.transfer_from(
            &e.current_contract_address(),
            &operator,
            &e.current_contract_address(),
            &adjust_amount(&e, amount),
        );

        event::deposit(&e, operator, e.current_contract_address(), amount);
        Ok(())
    }

    fn withdraw_and_deposit(
        e: Env,
        deposit_amount: i128,
        withdraw_amount: i128,
    ) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        let operator = read_operator(&e);
        if operator.is_none() {
            return Err(PumpBTCStakingError::NoOperatorSet);
        }
        let operator = operator.unwrap();
        check_operator(&e, &operator)?;

        let asset_token = read_asset_token_address(&e);
        let asset_client = token::Client::new(&e, &asset_token);

        check_nonnegative_amount(deposit_amount)?;

        let old_pending_stake_amount = read_pending_stake_amount(&e);
        write_pending_stake_amount(&e, 0);

        let total_claimable_amount = read_total_claimable_amount(&e);
        write_total_claimable_amount(&e, total_claimable_amount + deposit_amount);

        event::withdraw(&e, operator.clone(), withdraw_amount);
        event::deposit(
            &e,
            operator.clone(),
            e.current_contract_address(),
            deposit_amount,
        );

        if old_pending_stake_amount > deposit_amount {
            asset_client.transfer(
                &e.current_contract_address(),
                &operator,
                &adjust_amount(&e, withdraw_amount),
            );
        } else if old_pending_stake_amount < deposit_amount {
            asset_client.transfer_from(
                &e.current_contract_address(),
                &operator,
                &e.current_contract_address(),
                &adjust_amount(&e, deposit_amount),
            );
        }

        Ok(())
    }

    // // ========================= 用户功能 =========================

    // pub fn stake(e: Env, user: Address, amount: i128) {
    //     user.require_auth();

    //     check_nonnegative_amount(amount);
    //     if amount <= 0 {
    //         panic!("amount should be greater than 0");
    //     }

    //     let current_staking = read_total_staking_amount(&e);
    //     let staking_cap = read_total_staking_cap(&e);

    //     if current_staking + amount > staking_cap {
    //         panic!("exceed staking cap");
    //     }

    //     // 更新状态
    //     write_total_staking_amount(&e, current_staking + amount);
    //     let current_pending = read_pending_stake_amount(&e);
    //     write_pending_stake_amount(&e, current_pending + amount);

    //     // 转移资产代币到合约
    //     let asset_token = read_asset_token_address(&e);
    //     let asset_client = token::Client::new(&e, &asset_token);
    //     asset_client.transfer_from(
    //         &e.current_contract_address(),
    //         &user,
    //         &e.current_contract_address(),
    //         &amount,
    //     );

    //     // 铸造pumpBTC给用户 - 调用我们自定义的mint函数
    //     let pump_token = read_pump_token_address(&e);
    //     e.invoke_contract::<()>(
    //         &pump_token,
    //         &Symbol::new(&e, "mint"),
    //         (user.clone(), amount).into_val(&e),
    //     );

    //     e.events().publish((symbol_short!("stake"), user), amount);
    // }

    // pub fn unstake_request(e: Env, user: Address, amount: i128) {
    //     user.require_auth();
    //     check_unstake_allowed(&e);

    //     check_nonnegative_amount(amount);
    //     if amount <= 0 {
    //         panic!("amount should be greater than 0");
    //     }

    //     let current_time = e.ledger().timestamp();
    //     let slot = get_date_slot(current_time);

    //     // 检查是否可以在这个槽位请求解质押
    //     let last_request_time = read_pending_unstake_time(&e, &user, slot);
    //     let current_pending = read_pending_unstake_amount(&e, &user, slot);

    //     const SECONDS_PER_DAY: u64 = 86400;
    //     if current_time - last_request_time < SECONDS_PER_DAY && current_pending > 0 {
    //         panic!("claim the previous unstake first");
    //     }

    //     // 更新状态
    //     write_pending_unstake_time(&e, &user, slot, current_time);
    //     write_pending_unstake_amount(&e, &user, slot, current_pending + amount);

    //     let current_staking = read_total_staking_amount(&e);
    //     write_total_staking_amount(&e, current_staking - amount);

    //     let current_requested = read_total_requested_amount(&e);
    //     write_total_requested_amount(&e, current_requested + amount);

    //     // 销毁用户的pumpBTC
    //     let pump_token = read_pump_token_address(&e);
    //     e.invoke_contract::<()>(
    //         &pump_token,
    //         &Symbol::new(&e, "burn"),
    //         (user.clone(), amount).into_val(&e),
    //     );

    //     e.events()
    //         .publish((symbol_short!("unstake"), user, slot), amount);
    // }

    // pub fn claim_slot(e: Env, user: Address, slot: u32) -> i128 {
    //     user.require_auth();
    //     check_unstake_allowed(&e);

    //     if slot >= MAX_DATE_SLOT {
    //         panic!("invalid slot");
    //     }

    //     let amount = read_pending_unstake_amount(&e, &user, slot);
    //     if amount <= 0 {
    //         panic!("no pending unstake");
    //     }

    //     let request_time = read_pending_unstake_time(&e, &user, slot);
    //     let current_time = e.ledger().timestamp();

    //     const SECONDS_PER_DAY: u64 = 86400;
    //     let required_wait = (MAX_DATE_SLOT - 1) as u64 * SECONDS_PER_DAY;

    //     if current_time - request_time < required_wait {
    //         panic!("haven't reached the claimable time");
    //     }

    //     // 计算费用
    //     let fee_rate = read_normal_unstake_fee(&e);
    //     let fee = amount * fee_rate / 10000;
    //     let net_amount = amount - fee;

    //     // 清空该槽位
    //     write_pending_unstake_amount(&e, &user, slot, 0);

    //     // 更新总量
    //     let current_claimable = read_total_claimable_amount(&e);
    //     write_total_claimable_amount(&e, current_claimable - amount);

    //     let current_requested = read_total_requested_amount(&e);
    //     write_total_requested_amount(&e, current_requested - amount);

    //     let current_fee = read_collected_fee(&e);
    //     write_collected_fee(&e, current_fee + fee);

    //     // 转移资产给用户
    //     let asset_token = read_asset_token_address(&e);
    //     let asset_client = token::Client::new(&e, &asset_token);
    //     asset_client.transfer(&e.current_contract_address(), &user, &net_amount);

    //     e.events()
    //         .publish((symbol_short!("claim"), user, slot), net_amount);

    //     net_amount
    // }

    // pub fn unstake_instant(e: Env, user: Address, amount: i128) -> i128 {
    //     user.require_auth();
    //     check_unstake_allowed(&e);

    //     check_nonnegative_amount(amount);
    //     if amount <= 0 {
    //         panic!("amount should be greater than 0");
    //     }

    //     let pending_stake = read_pending_stake_amount(&e);
    //     if amount > pending_stake {
    //         panic!("insufficient pending stake amount");
    //     }

    //     // 计算费用
    //     let fee_rate = read_instant_unstake_fee(&e);
    //     let fee = amount * fee_rate / 10000;
    //     let net_amount = amount - fee;

    //     // 更新状态
    //     let current_staking = read_total_staking_amount(&e);
    //     write_total_staking_amount(&e, current_staking - amount);
    //     write_pending_stake_amount(&e, pending_stake - amount);

    //     let current_fee = read_collected_fee(&e);
    //     write_collected_fee(&e, current_fee + fee);

    //     // 销毁用户的pumpBTC
    //     let pump_token = read_pump_token_address(&e);
    //     e.invoke_contract::<()>(
    //         &pump_token,
    //         &Symbol::new(&e, "burn"),
    //         (user.clone(), amount).into_val(&e),
    //     );

    //     // 转移资产给用户
    //     let asset_token = read_asset_token_address(&e);
    //     let asset_client = token::Client::new(&e, &asset_token);
    //     asset_client.transfer(&e.current_contract_address(), &user, &net_amount);

    //     e.events()
    //         .publish((symbol_short!("instant"), user), net_amount);

    //     net_amount
    // }

    // // ========================= 查询函数 =========================

    // pub fn get_staking_info(e: Env) -> (i128, i128, i128, i128, i128) {
    //     (
    //         read_total_staking_amount(&e),
    //         read_total_staking_cap(&e),
    //         read_total_requested_amount(&e),
    //         read_total_claimable_amount(&e),
    //         read_pending_stake_amount(&e),
    //     )
    // }

    // pub fn get_fees(e: Env) -> (i128, i128, i128) {
    //     (
    //         read_normal_unstake_fee(&e),
    //         read_instant_unstake_fee(&e),
    //         read_collected_fee(&e),
    //     )
    // }

    // pub fn get_pending_unstake(e: Env, user: Address, slot: u32) -> (i128, u64) {
    //     (
    //         read_pending_unstake_amount(&e, &user, slot),
    //         read_pending_unstake_time(&e, &user, slot),
    //     )
    // }

    // pub fn get_addresses(e: Env) -> (Address, Address, Option<Address>) {
    //     (
    //         read_pump_token_address(&e),
    //         read_asset_token_address(&e),
    //         read_operator(&e),
    //     )
    // }
}
