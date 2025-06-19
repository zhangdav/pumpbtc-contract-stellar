use crate::admin::{has_administrator, read_administrator, write_administrator};
use crate::error::PumpBTCStakingError;
use crate::event;
use crate::storage::*;
use crate::storage_types::{MAX_DATE_SLOT, SECONDS_PER_DAY, UTC_OFFSET};
use soroban_sdk::{contract, contractimpl, token, Address, Env, IntoVal, Symbol};

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

    // ========================= User Functions =========================

    fn stake(e: Env, user: Address, amount: i128) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        user.require_auth();
        check_nonnegative_amount(amount)?;

        let total_staking_amount = read_total_staking_amount(&e);
        let total_staking_cap = read_total_staking_cap(&e);

        if total_staking_amount + amount > total_staking_cap {
            return Err(PumpBTCStakingError::ExceedStakingCap);
        }

        write_total_staking_amount(&e, total_staking_amount + amount);
        let pending_stake_amount = read_pending_stake_amount(&e);
        write_pending_stake_amount(&e, pending_stake_amount + amount);

        let asset_token = read_asset_token_address(&e);
        let asset_client = token::Client::new(&e, &asset_token);

        asset_client.transfer_from(
            &e.current_contract_address(),
            &user,
            &e.current_contract_address(),
            &adjust_amount(&e, amount),
        );

        // Mint pumpBTC to user
        let pump_token = read_pump_token_address(&e);
        e.invoke_contract::<()>(
            &pump_token,
            &Symbol::new(&e, "mint"),
            (user.clone(), amount).into_val(&e),
        );

        event::stake(&e, user, amount);
        Ok(())
    }

    fn unstake_request(e: Env, user: Address, amount: i128) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        user.require_auth();
        check_unstake_allowed(&e)?;

        check_nonnegative_amount(amount)?;

        let block_timestamp = e.ledger().timestamp();
        let slot = get_date_slot(block_timestamp);

        // Check if the user can request unstake in this slot
        let pending_unstake_time = read_pending_unstake_time(&e, &user, slot);
        let pending_unstake_amount = read_pending_unstake_amount(&e, &user, slot);

        if block_timestamp - pending_unstake_time < SECONDS_PER_DAY && pending_unstake_amount > 0 {
            return Err(PumpBTCStakingError::ClaimPreviousUnstakeFirst);
        }

        write_pending_unstake_time(&e, &user, slot, block_timestamp);
        write_pending_unstake_amount(&e, &user, slot, pending_unstake_amount + amount);

        let total_staking_amount = read_total_staking_amount(&e);
        write_total_staking_amount(&e, total_staking_amount - amount);

        let total_requested_amount = read_total_requested_amount(&e);
        write_total_requested_amount(&e, total_requested_amount + amount);

        // Burn user's pumpBTC
        let pump_token = read_pump_token_address(&e);
        e.invoke_contract::<()>(
            &pump_token,
            &Symbol::new(&e, "burn"),
            (user.clone(), amount).into_val(&e),
        );

        event::unstake_request(&e, user, amount, slot);
        Ok(())
    }

    fn claim_slot(e: Env, user: Address, slot: u32) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        user.require_auth();
        check_unstake_allowed(&e)?;

        let amount = read_pending_unstake_amount(&e, &user, slot);
        let normal_unstake_fee = read_normal_unstake_fee(&e);
        let fee = amount * normal_unstake_fee / 10000;

        check_nonnegative_amount(fee)?;

        let block_timestamp = e.ledger().timestamp();
        let pending_unstake_time = read_pending_unstake_time(&e, &user, slot);

        if block_timestamp - pending_unstake_time >= (MAX_DATE_SLOT - 1) as u64 * SECONDS_PER_DAY {
            return Err(PumpBTCStakingError::NotReachedClaimableTime);
        }

        write_pending_unstake_amount(&e, &user, slot, 0);

        let total_claimable_amount = read_total_claimable_amount(&e);
        write_total_claimable_amount(&e, total_claimable_amount - amount);

        let total_requested_amount = read_total_requested_amount(&e);
        write_total_requested_amount(&e, total_requested_amount - amount);

        let collected_fee = read_collected_fee(&e);
        write_collected_fee(&e, collected_fee + fee);

        let asset_token = read_asset_token_address(&e);
        let asset_client = token::Client::new(&e, &asset_token);

        asset_client.transfer(&e.current_contract_address(), &user, &adjust_amount(&e, amount - fee));

        event::claim_slot(&e, user, amount, slot);
        Ok(())
    }

    fn claim_all(e: Env, user: Address) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);
        Ok(())
    }

    fn unstake_instant(e: Env, user: Address, amount: i128) -> Result<(), PumpBTCStakingError> {
        extend_instance_ttl(&e);

        user.require_auth();
        check_unstake_allowed(&e)?;

        let collected_fee = read_collected_fee(&e);
        let fee = amount * collected_fee / 10000;

        check_nonnegative_amount(amount)?;

        let pending_stake_amount = read_pending_stake_amount(&e);
        if amount > pending_stake_amount {
            return Err(PumpBTCStakingError::InsufficientPendingStakeAmount);
        }

        let total_staking_amount = read_total_staking_amount(&e);
        write_total_staking_amount(&e, total_staking_amount - amount);

        write_pending_stake_amount(&e, pending_stake_amount - amount);

        write_collected_fee(&e, collected_fee + fee);

        // Burn user's pumpBTC
        let pump_token = read_pump_token_address(&e);
        e.invoke_contract::<()>(
            &pump_token,
            &Symbol::new(&e, "burn"),
            (user.clone(), amount).into_val(&e),
        );

        let asset_token = read_asset_token_address(&e);
        let asset_client = token::Client::new(&e, &asset_token);

        asset_client.transfer(&e.current_contract_address(), &user, &adjust_amount(&e, amount - fee));

        event::unstake_instant(&e, user, amount);
        Ok(())
    }

    // // ========================= Getter Functions =========================

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
