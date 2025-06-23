use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, IntoVal,
};

use crate::tests::test_setup::{PumpBTCStakingTest, pumpbtc_staking, STAKING_AMOUNT, DEPOSIT_AMOUNT, EXPIRATION_LEDGER};

#[test]
fn test_transfer_admin() {
    let test = PumpBTCStakingTest::setup_initialized();
    let new_admin = Address::generate(&test.env);

    test.contract.transfer_admin(&new_admin);
    assert_eq!(Some(new_admin.clone()), test.contract.get_pending_admin());

    test.contract.accept_admin();
    assert_eq!(None, test.contract.get_pending_admin());
}

#[test]
fn test_renounce_admin() {
    let test = PumpBTCStakingTest::setup_initialized();
    test.contract.renounce_admin();
    assert_eq!(None, test.contract.get_pending_admin());
}

#[test]
fn test_pause_unpause() {
    let test = PumpBTCStakingTest::setup_initialized();
    assert_eq!(false, test.contract.is_paused());

    test.contract.pause();
    assert_eq!(true, test.contract.is_paused());

    let result = test.contract.try_stake(&test.user1, &STAKING_AMOUNT);
    assert_eq!(
        result,
        Err(Ok(pumpbtc_staking::PumpBTCStakingError::ContractIsPaused))
    );

    test.contract.unpause();
    assert_eq!(false, test.contract.is_paused());

    test.asset_token.approve(
        &test.user1,
        &test.contract.address,
        &STAKING_AMOUNT,
        &EXPIRATION_LEDGER,
    );

    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "stake",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    STAKING_AMOUNT.into_val(&test.env),
                ],
                sub_invokes: &[
                    MockAuthInvoke {
                        contract: &test.asset_token.address,
                        fn_name: "transfer_from",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.contract.address.into_val(&test.env), // spender
                            test.user1.into_val(&test.env),            // from
                            test.contract.address.into_val(&test.env), // to
                            STAKING_AMOUNT.into_val(&test.env),        // amount
                        ],
                        sub_invokes: &[],
                    },
                    MockAuthInvoke {
                        contract: &test.pump_token.address,
                        fn_name: "mint",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.user1.into_val(&test.env),     // to
                            STAKING_AMOUNT.into_val(&test.env), // amount
                        ],
                        sub_invokes: &[],
                    },
                ],
            },
        }])
        .stake(&test.user1, &STAKING_AMOUNT);

    assert_eq!(STAKING_AMOUNT, test.contract.get_pending_stake_amount());
}

#[test]
fn test_set_stake_asset_cap() {
    let test = PumpBTCStakingTest::setup_initialized();
    assert_eq!(10_000_000_000i128, test.contract.get_total_staking_cap());

    let new_cap = 50_000_000_000i128;

    test.contract.set_stake_asset_cap(&new_cap);
    assert_eq!(new_cap, test.contract.get_total_staking_cap());
}

#[test]
fn test_set_normal_unstake_fee() {
    let test = PumpBTCStakingTest::setup_initialized();
    assert_eq!(100i128, test.contract.get_normal_unstake_fee()); // 1%

    let new_fee = 500i128; // 5%

    test.contract.set_normal_unstake_fee(&new_fee);
    assert_eq!(new_fee, test.contract.get_normal_unstake_fee());
}

#[test]
fn test_set_instant_unstake_fee() {
    let test = PumpBTCStakingTest::setup_initialized();
    assert_eq!(500i128, test.contract.get_instant_unstake_fee()); // 5%

    let new_fee = 1000i128; // 10%

    test.contract.set_instant_unstake_fee(&new_fee);
    assert_eq!(new_fee, test.contract.get_instant_unstake_fee());
}

#[test]
fn test_set_operator() {
    let test = PumpBTCStakingTest::setup_initialized();
    assert_eq!(Some(test.operator), test.contract.get_operator());

    let new_operator = Address::generate(&test.env);

    test.contract.set_operator(&new_operator);
    assert_eq!(Some(new_operator.clone()), test.contract.get_operator());
}

#[test]
fn test_set_only_allow_stake() {
    let test = PumpBTCStakingTest::setup_initialized();
    assert_eq!(true, test.contract.get_only_allow_stake());

    test.contract.set_only_allow_stake(&false);
    assert_eq!(false, test.contract.get_only_allow_stake());
}

#[test]
fn test_deposit() {
    let test = PumpBTCStakingTest::setup_initialized();
    test.asset_token.approve(
        &test.operator,
        &test.contract.address,
        &DEPOSIT_AMOUNT,
        &EXPIRATION_LEDGER,
    );
    test.contract.deposit(&DEPOSIT_AMOUNT);

    assert_eq!(DEPOSIT_AMOUNT, test.contract.get_total_claimable_amount());
}

#[test]
fn test_withdraw() {
    let test = PumpBTCStakingTest::setup_initialized();
    test.asset_token.approve(
        &test.user1,
        &test.contract.address,
        &STAKING_AMOUNT,
        &EXPIRATION_LEDGER,
    );

    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "stake",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    STAKING_AMOUNT.into_val(&test.env),
                ],
                sub_invokes: &[
                    MockAuthInvoke {
                        contract: &test.asset_token.address,
                        fn_name: "transfer_from",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.contract.address.into_val(&test.env), // spender
                            test.user1.into_val(&test.env),            // from
                            test.contract.address.into_val(&test.env), // to
                            STAKING_AMOUNT.into_val(&test.env),        // amount
                        ],
                        sub_invokes: &[],
                    },
                    MockAuthInvoke {
                        contract: &test.pump_token.address,
                        fn_name: "mint",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.user1.into_val(&test.env),     // to
                            STAKING_AMOUNT.into_val(&test.env), // amount
                        ],
                        sub_invokes: &[],
                    },
                ],
            },
        }])
        .stake(&test.user1, &STAKING_AMOUNT);

    assert_eq!(STAKING_AMOUNT, test.contract.get_pending_stake_amount());

    test.contract.withdraw();

    assert_eq!(0i128, test.contract.get_pending_stake_amount());
}

#[test]
fn test_withdraw_and_deposit() {
    let test = PumpBTCStakingTest::setup_initialized();

    test.asset_token.approve(
        &test.operator,
        &test.contract.address,
        &DEPOSIT_AMOUNT,
        &EXPIRATION_LEDGER,
    );
    test.contract.deposit(&DEPOSIT_AMOUNT);

    assert_eq!(DEPOSIT_AMOUNT, test.contract.get_total_claimable_amount());

    test.asset_token.approve(
        &test.operator,
        &test.contract.address,
        &DEPOSIT_AMOUNT,
        &EXPIRATION_LEDGER,
    );
    test.contract.withdraw_and_deposit(&DEPOSIT_AMOUNT);

    assert_eq!(
        DEPOSIT_AMOUNT + DEPOSIT_AMOUNT,
        test.contract.get_total_claimable_amount()
    );
}