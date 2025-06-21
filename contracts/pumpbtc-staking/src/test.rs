#![cfg(test)]
extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

/// TODO: need module this unit test

pub const STAKING_AMOUNT: i128 = 100_000_000;
pub const DEPOSIT_AMOUNT: i128 = 1_000_000_000;
pub const EXPIRATION_LEDGER: u32 = 99999;

// Create Asset token contract
mod asset_token {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/asset_token.wasm");
    pub type AssetTokenClient<'a> = Client<'a>;
}
use asset_token::AssetTokenClient;

fn create_asset_token_contract<'a>(e: &Env, admin: &Address) -> AssetTokenClient<'a> {
    let contract_address = e.register_contract_wasm(None, asset_token::WASM);
    let asset_token = AssetTokenClient::new(e, &contract_address);
    asset_token.initialize(admin, &8, &"WBTC".into_val(e), &"WBTC".into_val(e));
    asset_token
}

// Create Pump token contract
mod pump_token {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/pump_token.wasm");
    pub type PumpTokenClient<'a> = Client<'a>;
}
use pump_token::PumpTokenClient;

fn create_pump_token_contract<'a>(
    e: &Env,
    admin: &Address,
    minter: &Address,
) -> PumpTokenClient<'a> {
    let contract_address = e.register_contract_wasm(None, pump_token::WASM);
    let pump_token = PumpTokenClient::new(e, &contract_address);

    // Initialize the pump token contract
    pump_token.initialize(
        admin,
        minter,
        &"pumpBTC".into_val(e),
        &"pumpBTC".into_val(e),
    );

    pump_token
}

// Create PumpBTCStaking contract
mod pumpbtc_staking {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/pumpbtc_staking.wasm");
    pub type PumpBTCStakingClient<'a> = Client<'a>;
}
use pumpbtc_staking::PumpBTCStakingClient;

fn create_pumpbtc_staking_contract<'a>(e: &Env) -> PumpBTCStakingClient<'a> {
    let pumpbtc_staking_address = &e.register_contract_wasm(None, pumpbtc_staking::WASM);
    let pumpbtc_staking = PumpBTCStakingClient::new(e, pumpbtc_staking_address);
    pumpbtc_staking
}

// PumpBTCStaking TEST Structure
pub struct PumpBTCStakingTest<'a> {
    pub env: Env,
    pub contract: PumpBTCStakingClient<'a>,
    pub pump_token: PumpTokenClient<'a>,
    pub asset_token: AssetTokenClient<'a>,
    pub admin: Address,
    pub user1: Address,
    pub user2: Address,
    pub operator: Address,
}

impl<'a> PumpBTCStakingTest<'a> {
    pub fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let operator = Address::generate(&env);

        let contract = create_pumpbtc_staking_contract(&env);

        let pump_token_contract = create_pump_token_contract(&env, &admin, &contract.address);
        let asset_token_contract = create_asset_token_contract(&env, &admin);

        asset_token_contract.mint(&user1, &10_000_000_000_000_000_000);
        asset_token_contract.mint(&user2, &10_000_000_000_000_000_000);
        asset_token_contract.mint(&admin, &10_000_000_000_000_000_000);
        asset_token_contract.mint(&operator, &10_000_000_000_000_000_000);

        pump_token_contract.mint(&user1, &10_000_000_000_000_000_000);
        pump_token_contract.mint(&user2, &10_000_000_000_000_000_000);
        pump_token_contract.mint(&admin, &10_000_000_000_000_000_000);

        env.cost_estimate().budget().reset_unlimited();

        PumpBTCStakingTest {
            env,
            contract,
            pump_token: pump_token_contract,
            asset_token: asset_token_contract,
            admin,
            user1,
            user2,
            operator,
        }
    }

    pub fn setup_initialized() -> Self {
        let test = Self::setup();
        test.contract.initialize(
            &test.admin,
            &test.pump_token.address,
            &test.asset_token.address,
        );

        test.contract.set_stake_asset_cap(&(10_000_000_000i128));
        test.contract.set_operator(&test.operator);
        test.contract.set_normal_unstake_fee(&100); // 1%
        test.contract.set_instant_unstake_fee(&500); // 5%

        test
    }
}

#[test]
fn test_initialize() {
    let test = PumpBTCStakingTest::setup_initialized();

    assert_eq!(test.pump_token.address, test.contract.get_pump_token());
    assert_eq!(test.asset_token.address, test.contract.get_asset_token());
    assert_eq!(8u32, test.contract.get_asset_decimal());
    assert_eq!(100i128, test.contract.get_normal_unstake_fee());
    assert_eq!(500i128, test.contract.get_instant_unstake_fee());
    assert_eq!(true, test.contract.get_only_allow_stake());
}

#[test]
#[should_panic]
fn test_initialize_twice() {
    let test = PumpBTCStakingTest::setup_initialized();
    test.contract.initialize(
        &test.admin,
        &test.pump_token.address,
        &test.asset_token.address,
    );
}

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
fn test_getter_functions() {
    let test = PumpBTCStakingTest::setup_initialized();

    assert_eq!(test.pump_token.address, test.contract.get_pump_token());
    assert_eq!(test.asset_token.address, test.contract.get_asset_token());
    assert_eq!(8u32, test.contract.get_asset_decimal());
    assert_eq!(0i128, test.contract.get_total_staking_amount());
    assert_eq!(100_000_000_00i128, test.contract.get_total_staking_cap());
    assert_eq!(0i128, test.contract.get_total_requested_amount());
    assert_eq!(0i128, test.contract.get_total_claimable_amount());
    assert_eq!(0i128, test.contract.get_pending_stake_amount());
    assert_eq!(0i128, test.contract.get_collected_fee());
    assert_eq!(Some(test.operator), test.contract.get_operator());
    assert_eq!(100i128, test.contract.get_normal_unstake_fee());
    assert_eq!(500i128, test.contract.get_instant_unstake_fee());
    assert_eq!(true, test.contract.get_only_allow_stake());
    assert_eq!(false, test.contract.is_paused());
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
