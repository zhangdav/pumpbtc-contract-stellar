use soroban_sdk::{
    testutils::{Ledger, MockAuth, MockAuthInvoke},
    IntoVal,
};

use crate::tests::test_setup::{PumpBTCStakingTest, pumpbtc_staking, STAKING_AMOUNT, EXPIRATION_LEDGER};

/// TODO: need add edge case test and error test
#[test]
fn test_stake() {
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
                            STAKING_AMOUNT.into_val(&test.env),
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

    assert_eq!(STAKING_AMOUNT, test.contract.get_total_staking_amount());
}

#[test]
fn test_unstake_request() {
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
                            STAKING_AMOUNT.into_val(&test.env),
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

    test.contract.set_only_allow_stake(&false);

    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "unstake_request",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    STAKING_AMOUNT.into_val(&test.env),
                ],
                sub_invokes: &[MockAuthInvoke {
                    contract: &test.pump_token.address,
                    fn_name: "burn",
                    args: soroban_sdk::vec![
                        &test.env,
                        test.user1.into_val(&test.env),     // from
                        STAKING_AMOUNT.into_val(&test.env), // amount
                    ],
                    sub_invokes: &[],
                }],
            },
        }])
        .unstake_request(&test.user1, &STAKING_AMOUNT);

    assert_eq!(STAKING_AMOUNT, test.contract.get_total_requested_amount());
}

#[test]
fn test_unstake_instant() {
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
                            STAKING_AMOUNT.into_val(&test.env),
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

    test.contract.set_only_allow_stake(&false);

    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "unstake_instant",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    STAKING_AMOUNT.into_val(&test.env),
                ],
                sub_invokes: &[
                    MockAuthInvoke {
                        contract: &test.pump_token.address,
                        fn_name: "burn",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.user1.into_val(&test.env),     // from
                            STAKING_AMOUNT.into_val(&test.env), // amount
                        ],
                        sub_invokes: &[],
                    },
                    MockAuthInvoke {
                        contract: &test.asset_token.address,
                        fn_name: "transfer",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.contract.address.into_val(&test.env), // from
                            test.user1.into_val(&test.env),            // to
                            STAKING_AMOUNT.into_val(&test.env),        // amount
                        ],
                        sub_invokes: &[],
                    },
                ],
            },
        }])
        .unstake_instant(&test.user1, &STAKING_AMOUNT);

    assert_eq!(0i128, test.contract.get_total_staking_amount());
}

#[test]
fn test_collect_fee() {
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
                            STAKING_AMOUNT.into_val(&test.env),
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

    test.contract.set_only_allow_stake(&false);

    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "unstake_instant",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    STAKING_AMOUNT.into_val(&test.env),
                ],
                sub_invokes: &[
                    MockAuthInvoke {
                        contract: &test.pump_token.address,
                        fn_name: "burn",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.user1.into_val(&test.env),     // from
                            STAKING_AMOUNT.into_val(&test.env), // amount
                        ],
                        sub_invokes: &[],
                    },
                    MockAuthInvoke {
                        contract: &test.asset_token.address,
                        fn_name: "transfer",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.contract.address.into_val(&test.env), // from
                            test.user1.into_val(&test.env),            // to
                            STAKING_AMOUNT.into_val(&test.env),        // amount
                        ],
                        sub_invokes: &[],
                    },
                ],
            },
        }])
        .unstake_instant(&test.user1, &STAKING_AMOUNT);

    assert_eq!(0i128, test.contract.get_total_staking_amount());

    assert_eq!(
        STAKING_AMOUNT * 500 / 10000,
        test.contract.get_collected_fee()
    );

    test.contract.collect_fee();

    assert_eq!(0i128, test.contract.get_collected_fee());
}

#[test]
fn test_claim_slot() {
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
                            STAKING_AMOUNT.into_val(&test.env),
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

    test.contract.set_only_allow_stake(&false);

    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "unstake_request",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    STAKING_AMOUNT.into_val(&test.env),
                ],
                sub_invokes: &[MockAuthInvoke {
                    contract: &test.pump_token.address,
                    fn_name: "burn",
                    args: soroban_sdk::vec![
                        &test.env,
                        test.user1.into_val(&test.env),     // from
                        STAKING_AMOUNT.into_val(&test.env), // amount
                    ],
                    sub_invokes: &[],
                }],
            },
        }])
        .unstake_request(&test.user1, &STAKING_AMOUNT);

    let slot = test.env.ledger().timestamp() as u32;
    let pending_amount = test.contract.get_pending_unstake_amount(&test.user1, &slot);

    assert_eq!(STAKING_AMOUNT, pending_amount);

    test.env.ledger().with_mut(|ledger| {
        ledger.timestamp = ledger.timestamp + 9 * 24 * 60 * 60; // 9 days
    });

    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "claim_slot",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    slot.into_val(&test.env),
                ],
                sub_invokes: &[MockAuthInvoke {
                    contract: &test.asset_token.address,
                    fn_name: "transfer",
                    args: soroban_sdk::vec![
                        &test.env,
                        test.contract.address.into_val(&test.env), // from
                        test.user1.into_val(&test.env),            // to
                        STAKING_AMOUNT.into_val(&test.env),        // amount
                    ],
                    sub_invokes: &[],
                }],
            },
        }])
        .claim_slot(&test.user1, &slot);

    assert_eq!(0i128, test.contract.get_total_staking_amount());
    assert_eq!(0i128, test.contract.get_total_requested_amount());

    assert_eq!(
        0i128,
        test.contract.get_pending_unstake_amount(&test.user1, &slot)
    );
}

#[test]
fn test_claim_all() {
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
                            STAKING_AMOUNT.into_val(&test.env),
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

    test.contract.set_only_allow_stake(&false);

    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "unstake_request",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    STAKING_AMOUNT.into_val(&test.env),
                ],
                sub_invokes: &[MockAuthInvoke {
                    contract: &test.pump_token.address,
                    fn_name: "burn",
                    args: soroban_sdk::vec![
                        &test.env,
                        test.user1.into_val(&test.env),     // from
                        STAKING_AMOUNT.into_val(&test.env), // amount
                    ],
                    sub_invokes: &[],
                }],
            },
        }])
        .unstake_request(&test.user1, &STAKING_AMOUNT);

    test.env.ledger().with_mut(|ledger| {
        ledger.timestamp = ledger.timestamp + 9 * 24 * 60 * 60; // 9 days
    });

    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "claim_all",
                args: soroban_sdk::vec![&test.env, test.user1.into_val(&test.env),],
                sub_invokes: &[MockAuthInvoke {
                    contract: &test.asset_token.address,
                    fn_name: "transfer",
                    args: soroban_sdk::vec![
                        &test.env,
                        test.contract.address.into_val(&test.env), // from
                        test.user1.into_val(&test.env),            // to
                        STAKING_AMOUNT.into_val(&test.env),        // amount
                    ],
                    sub_invokes: &[],
                }],
            },
        }])
        .claim_all(&test.user1);

    assert_eq!(0i128, test.contract.get_total_requested_amount());
}

#[test]
fn test_stake_exceeds_cap() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = STAKING_AMOUNT * 100000 as i128;

    test.asset_token.approve(
        &test.user1,
        &test.contract.address,
        &stake_amount,
        &EXPIRATION_LEDGER,
    );

    let result = test.contract.try_stake(&test.user1, &stake_amount);
    assert_eq!(
        result,
        Err(Ok(pumpbtc_staking::PumpBTCStakingError::ExceedStakingCap))
    );
}

#[test]
fn test_unstake_when_only_allow_stake() {
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
                            STAKING_AMOUNT.into_val(&test.env),
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

    let result = test
        .contract
        .try_unstake_request(&test.user1, &STAKING_AMOUNT);
    assert_eq!(
        result,
        Err(Ok(
            pumpbtc_staking::PumpBTCStakingError::OnlyAllowStakeAtFirst
        ))
    );
}

#[test]
fn test_withdraw_without_pending_amount() {
    let test = PumpBTCStakingTest::setup_initialized();

    let result = test.contract.try_withdraw();
    assert_eq!(
        result,
        Err(Ok(
            pumpbtc_staking::PumpBTCStakingError::NoPendingStakeAmount
        ))
    );
}

#[test]
fn test_collect_fee_without_fee() {
    let test = PumpBTCStakingTest::setup_initialized();

    let result = test.contract.try_collect_fee();
    assert_eq!(
        result,
        Err(Ok(pumpbtc_staking::PumpBTCStakingError::NoFeeToCollect))
    );
}

#[test]
fn test_set_invalid_fee() {
    let test = PumpBTCStakingTest::setup_initialized();

    let result = test.contract.try_set_instant_unstake_fee(&10001);
    assert_eq!(
        result,
        Err(Ok(
            pumpbtc_staking::PumpBTCStakingError::FeeShouldBeBetween0And10000
        ))
    );
}

#[test]
fn test_claim_slot_too_early() {
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
                            STAKING_AMOUNT.into_val(&test.env),
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

    test.contract.set_only_allow_stake(&false);

    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "unstake_request",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    STAKING_AMOUNT.into_val(&test.env),
                ],
                sub_invokes: &[MockAuthInvoke {
                    contract: &test.pump_token.address,
                    fn_name: "burn",
                    args: soroban_sdk::vec![
                        &test.env,
                        test.user1.into_val(&test.env),     // from
                        STAKING_AMOUNT.into_val(&test.env), // amount
                    ],
                    sub_invokes: &[],
                }],
            },
        }])
        .unstake_request(&test.user1, &STAKING_AMOUNT);

    let slot = test.env.ledger().timestamp() as u32;

    let result = test.contract.try_claim_slot(&test.user1, &slot);
    assert_eq!(
        result,
        Err(Ok(
            pumpbtc_staking::PumpBTCStakingError::NotReachedClaimableTime
        ))
    );
}

#[test]
fn test_pause_when_already_paused() {
    let test = PumpBTCStakingTest::setup_initialized();

    let result = test.contract.try_pause();
    assert_eq!(result, Ok(Ok(())));

    let result = test.contract.try_pause();
    assert_eq!(
        result,
        Err(Ok(pumpbtc_staking::PumpBTCStakingError::ContractIsPaused))
    );
}

#[test]
fn test_unpause_when_not_paused() {
    let test = PumpBTCStakingTest::setup_initialized();

    let result = test.contract.try_unpause();
    assert_eq!(
        result,
        Err(Ok(
            pumpbtc_staking::PumpBTCStakingError::ContractIsNotPaused
        ))
    );
}
