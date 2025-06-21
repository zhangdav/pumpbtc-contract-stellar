#![cfg(test)]
extern crate std;

use soroban_sdk::{
    Env, 
    Address, 
    IntoVal,
    testutils::{
        Address as _,
        Ledger,
        MockAuth,
        MockAuthInvoke,
    },
};

use crate::error::PumpBTCStakingError;

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

fn create_pump_token_contract<'a>(e: &Env, admin: &Address, minter: &Address) -> PumpTokenClient<'a> {
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

fn create_pumpbtc_staking_contract<'a>(e: & Env) -> PumpBTCStakingClient<'a> {
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
        test.contract.initialize(&test.admin, &test.pump_token.address, &test.asset_token.address);
        
        test.contract.set_stake_asset_cap(&(10_000_000_000i128));
        test.contract.set_operator(&test.operator);
        test.contract.set_normal_unstake_fee(&100);    // 1%
        test.contract.set_instant_unstake_fee(&500);   // 5%
        
        test
    }
}

// ========================= 初始化测试 =========================
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
    test.contract.initialize(&test.admin, &test.pump_token.address, &test.asset_token.address);
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

    let stake_amount = 100_000_000i128; // 1 WBTC

    let result = test.contract.try_stake(&test.user1, &stake_amount);
    assert_eq!(result, Err(Ok(pumpbtc_staking::PumpBTCStakingError::ContractIsPaused)));
}

// ========================= 配置函数测试 =========================
#[test]
fn test_set_stake_asset_cap() {
    let test = PumpBTCStakingTest::setup_initialized();
    let new_cap = 1_000_000_000_000_000_000i128;
    
    test.contract.set_stake_asset_cap(&new_cap);
    assert_eq!(new_cap, test.contract.get_total_staking_cap());
}

#[test]
fn test_set_normal_unstake_fee() {
    let test = PumpBTCStakingTest::setup_initialized();
    let new_fee = 100i128; // 1%
    
    test.contract.set_normal_unstake_fee(&new_fee);
    assert_eq!(new_fee, test.contract.get_normal_unstake_fee());
}

#[test]
fn test_set_instant_unstake_fee() {
    let test = PumpBTCStakingTest::setup_initialized();
    let new_fee = 500i128; // 5%
    
    test.contract.set_instant_unstake_fee(&new_fee);
    assert_eq!(new_fee, test.contract.get_instant_unstake_fee());
}

#[test]
fn test_set_operator() {
    let test = PumpBTCStakingTest::setup_initialized();
    
    test.contract.set_operator(&test.operator);
    assert_eq!(Some(test.operator.clone()), test.contract.get_operator());
}

#[test]
fn test_set_only_allow_stake() {
    let test = PumpBTCStakingTest::setup_initialized();
    
    test.contract.set_only_allow_stake(&false);
    assert_eq!(false, test.contract.get_only_allow_stake());
}

// ========================= 存取款测试 =========================
#[test]
fn test_deposit() {
    let test = PumpBTCStakingTest::setup_initialized();
    let deposit_amount = 1_000_000_000i128;
    
    // 设置操作员
    test.contract.set_operator(&test.operator);
    
    // 操作员批准合约提取资产（应该是operator自己的资产）
    test.asset_token.approve(&test.operator, &test.contract.address, &deposit_amount, &99999);
    
    // 存入资产
    test.contract.deposit(&deposit_amount);
    
    // 验证待领取金额（deposit更新的是total_claimable_amount）
    assert_eq!(deposit_amount, test.contract.get_total_claimable_amount());
}

#[test]
fn test_withdraw() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 1_000_000_000i128;

    // 设置质押上限，允许stake操作
    test.contract.set_stake_asset_cap(&(10_000_000_000i128));

    // 设置操作员（在stake之前设置）
    test.contract.set_operator(&test.operator);

    test.asset_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);

    // 使用mock_auths处理跨合约调用授权
    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "stake",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    stake_amount.into_val(&test.env),
                ],
                sub_invokes: &[
                    // asset_token.transfer_from 子调用
                    MockAuthInvoke {
                        contract: &test.asset_token.address,
                        fn_name: "transfer_from",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.contract.address.into_val(&test.env), // spender
                            test.user1.into_val(&test.env),            // from
                            test.contract.address.into_val(&test.env), // to
                            stake_amount.into_val(&test.env),          // amount
                        ],
                        sub_invokes: &[],
                    },
                    // pump_token.mint 子调用 - 需要minter授权
                    MockAuthInvoke {
                        contract: &test.pump_token.address,
                        fn_name: "mint",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.user1.into_val(&test.env),   // to
                            stake_amount.into_val(&test.env), // amount
                        ],
                        sub_invokes: &[],
                    },
                ],
            },
        }])
        .stake(&test.user1, &stake_amount);
    
    // 验证pending_stake_amount增加了
    assert_eq!(stake_amount, test.contract.get_pending_stake_amount());

    // 现在可以执行withdraw了（简化版本，不使用mock_auths）
    test.contract.withdraw();

    // 验证pending_stake_amount清零
    assert_eq!(0i128, test.contract.get_pending_stake_amount());
}

#[test]
fn test_withdraw_and_deposit() {
    let test = PumpBTCStakingTest::setup_initialized();
    let initial_amount = 1_000_000_000i128;
    let new_deposit_amount = 2_000_000_000i128;
    
    // 设置操作员
    test.contract.set_operator(&test.operator);
    
    // 先存入初始资产（这会更新total_claimable_amount）
    test.asset_token.approve(&test.operator, &test.contract.address, &initial_amount, &99999);
    test.contract.deposit(&initial_amount);
    
    // 验证初始存款
    assert_eq!(initial_amount, test.contract.get_total_claimable_amount());
    
    // 再次批准新的存款金额
    test.asset_token.approve(&test.operator, &test.contract.address, &new_deposit_amount, &99999);
    
    // 执行提取并存入新金额
    test.contract.withdraw_and_deposit(&new_deposit_amount);
    
    // withdraw_and_deposit会在现有基础上增加deposit_amount
    assert_eq!(initial_amount + new_deposit_amount, test.contract.get_total_claimable_amount());
}

// ========================= 质押测试 =========================
#[test]
fn test_stake() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 100_000_000i128; // 1 WBTC
    
    // 设置质押上限
    test.contract.set_stake_asset_cap(&(10_000_000_000i128));

    test.asset_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);
    
    // 使用mock_auths处理跨合约调用授权
    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "stake",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    stake_amount.into_val(&test.env),
                ],
                sub_invokes: &[
                    // asset_token.transfer_from 子调用
                    MockAuthInvoke {
                        contract: &test.asset_token.address,
                        fn_name: "transfer_from",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.contract.address.into_val(&test.env), // spender
                            test.user1.into_val(&test.env),            // from
                            test.contract.address.into_val(&test.env), // to
                            stake_amount.into_val(&test.env),          // amount
                        ],
                        sub_invokes: &[],
                    },
                    // pump_token.mint 子调用 - 需要minter授权
                    MockAuthInvoke {
                        contract: &test.pump_token.address,
                        fn_name: "mint",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.user1.into_val(&test.env),   // to
                            stake_amount.into_val(&test.env), // amount
                        ],
                        sub_invokes: &[],
                    },
                ],
            },
        }])
        .stake(&test.user1, &stake_amount);
    
    // 验证质押结果
    assert_eq!(stake_amount, test.contract.get_total_staking_amount());
}

// ========================= 解质押请求测试 =========================
#[test]
fn test_unstake_request() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 100_000_000i128;
    let unstake_amount = 50_000_000i128;
    
    // 先执行质押
    test.contract.set_stake_asset_cap(&(10_000_000_000i128));
    test.contract.set_operator(&test.operator);
    test.asset_token.approve(&test.operator, &test.contract.address, &stake_amount, &99999);
    test.contract.deposit(&stake_amount);
    test.pump_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);
    test.contract.stake(&test.user1, &stake_amount);
    
    // 允许解质押
    test.contract.set_only_allow_stake(&false);
    
    // 执行解质押请求
    test.contract.unstake_request(&test.user1, &unstake_amount);
    
    // 验证请求的解质押总量
    assert_eq!(unstake_amount, test.contract.get_total_requested_amount());
}

// ========================= 立即解质押测试 =========================
#[test]
fn test_unstake_instant() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 100_000_000i128;
    let unstake_amount = 50_000_000i128;
    
    // 先执行质押
    test.contract.set_stake_asset_cap(&(10_000_000_000i128));
    test.contract.set_operator(&test.operator);
    
    // 存入更多资产以支持立即解质押
    let total_deposit = stake_amount + unstake_amount;
    test.asset_token.approve(&test.operator, &test.contract.address, &total_deposit, &99999);
    test.contract.deposit(&total_deposit);
    
    test.pump_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);
    test.contract.stake(&test.user1, &stake_amount);
    
    // 允许解质押
    test.contract.set_only_allow_stake(&false);
    
    // 执行立即解质押
    test.contract.unstake_instant(&test.user1, &unstake_amount);
    
    // 验证解质押后的状态
    assert_eq!(stake_amount - unstake_amount, test.contract.get_total_staking_amount());
}

// ========================= 手续费测试 =========================
#[test]
fn test_collect_fee() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 100_000_000i128;
    let unstake_amount = 50_000_000i128;
    
    // 设置立即解质押手续费为5%
    test.contract.set_instant_unstake_fee(&500);
    
    // 执行质押和立即解质押以产生手续费
    test.contract.set_stake_asset_cap(&(10_000_000_000i128));
    test.contract.set_operator(&test.operator);

    test.asset_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);
    
    // 使用mock_auths处理跨合约调用授权
    test.contract
        .mock_auths(&[MockAuth {
            address: &test.user1,
            invoke: &MockAuthInvoke {
                contract: &test.contract.address,
                fn_name: "stake",
                args: soroban_sdk::vec![
                    &test.env,
                    test.user1.into_val(&test.env),
                    stake_amount.into_val(&test.env),
                ],
                sub_invokes: &[
                    // asset_token.transfer_from 子调用
                    MockAuthInvoke {
                        contract: &test.asset_token.address,
                        fn_name: "transfer_from",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.contract.address.into_val(&test.env), // spender
                            test.user1.into_val(&test.env),            // from
                            test.contract.address.into_val(&test.env), // to
                            stake_amount.into_val(&test.env),          // amount
                        ],
                        sub_invokes: &[],
                    },
                    // pump_token.mint 子调用 - 需要minter授权
                    MockAuthInvoke {
                        contract: &test.pump_token.address,
                        fn_name: "mint",
                        args: soroban_sdk::vec![
                            &test.env,
                            test.user1.into_val(&test.env),   // to
                            stake_amount.into_val(&test.env), // amount
                        ],
                        sub_invokes: &[],
                    },
                ],
            },
        }])
        .stake(&test.user1, &stake_amount);
    
    test.contract.set_only_allow_stake(&false);
    test.contract.unstake_instant(&test.user1, &unstake_amount);
    
    // 验证手续费累积
    let expected_fee = unstake_amount * 500 / 10000; // 5%
    assert_eq!(expected_fee, test.contract.get_collected_fee());
    
    // 收取手续费
    test.contract.collect_fee();
    
    // 验证手续费清零
    assert_eq!(0i128, test.contract.get_collected_fee());
}

// ========================= 时间相关测试 =========================
#[test]
fn test_claim_slot() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 100_000_000i128;
    let unstake_amount = 50_000_000i128;
    
    // 执行质押和解质押请求
    test.contract.set_stake_asset_cap(&(10_000_000_000i128));
    test.contract.set_operator(&test.operator);
    
    let total_deposit = stake_amount + unstake_amount;
    test.asset_token.approve(&test.operator, &test.contract.address, &total_deposit, &99999);
    test.contract.deposit(&total_deposit);
    
    test.pump_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);
    test.contract.stake(&test.user1, &stake_amount);
    
    test.contract.set_only_allow_stake(&false);
    test.contract.unstake_request(&test.user1, &unstake_amount);
    
    // 获取槽位信息
    let slot = test.contract.get_max_date_slot();
    let pending_amount = test.contract.get_pending_unstake_amount(&test.user1, &slot);
    assert_eq!(unstake_amount, pending_amount);
    
    // 模拟时间推进（7天后）
    test.env.ledger().with_mut(|ledger| {
        ledger.timestamp = ledger.timestamp + 7 * 24 * 60 * 60; // 7天后
    });
    
    // 操作员处理解质押请求
    test.asset_token.approve(&test.operator, &test.contract.address, &(total_deposit - unstake_amount), &99999);
    test.contract.withdraw_and_deposit(&(total_deposit - unstake_amount));
    
    // 用户领取解质押资产
    test.contract.claim_slot(&test.user1, &slot);
    
    // 验证待领取金额变化
    assert_eq!(0i128, test.contract.get_pending_unstake_amount(&test.user1, &slot));
}

// ========================= 查询函数测试 =========================
#[test]
fn test_getter_functions() {
    let test = PumpBTCStakingTest::setup_initialized();
    
    // 测试基本查询函数
    assert_eq!(test.pump_token.address, test.contract.get_pump_token());
    assert_eq!(test.asset_token.address, test.contract.get_asset_token());
    assert_eq!(8u32, test.contract.get_asset_decimal());
    assert_eq!(0i128, test.contract.get_total_staking_amount());
    assert_eq!(0i128, test.contract.get_total_staking_cap());
    assert_eq!(0i128, test.contract.get_total_requested_amount());
    assert_eq!(0i128, test.contract.get_total_claimable_amount());
    assert_eq!(0i128, test.contract.get_pending_stake_amount());
    assert_eq!(0i128, test.contract.get_collected_fee());
    assert_eq!(None, test.contract.get_operator());
    assert_eq!(100i128, test.contract.get_normal_unstake_fee());
    assert_eq!(500i128, test.contract.get_instant_unstake_fee());
    assert_eq!(true, test.contract.get_only_allow_stake());
    assert_eq!(false, test.contract.is_paused());
}

// ========================= 错误处理测试 =========================
#[test]
#[should_panic]
fn test_stake_exceeds_cap() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 200_000_000i128;
    let cap = 100_000_000i128;
    
    // 设置较小的质押上限
    test.contract.set_stake_asset_cap(&cap);
    
    // 设置操作员并存入资产
    test.contract.set_operator(&test.operator);
    test.asset_token.approve(&test.operator, &test.contract.address, &stake_amount, &99999);
    test.contract.deposit(&stake_amount);
    
    // 用户批准合约转移pump token
    test.pump_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);
    
    // 尝试质押超过上限的金额，应该失败
    test.contract.stake(&test.user1, &stake_amount);
}

#[test]
#[should_panic]
fn test_unstake_when_only_allow_stake() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 100_000_000i128;
    let unstake_amount = 50_000_000i128;
    
    // 先执行质押
    test.contract.set_stake_asset_cap(&(10_000_000_000i128));
    test.contract.set_operator(&test.operator);
    test.asset_token.approve(&test.operator, &test.contract.address, &stake_amount, &99999);
    test.contract.deposit(&stake_amount);
    test.pump_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);
    test.contract.stake(&test.user1, &stake_amount);
    
    // 保持only_allow_stake为true
    // 尝试解质押，应该失败
    test.contract.unstake_request(&test.user1, &unstake_amount);
}

#[test]
#[should_panic]
fn test_deposit_without_operator() {
    let test = PumpBTCStakingTest::setup_initialized();
    let deposit_amount = 1_000_000_000i128;
    
    // 不设置操作员，直接尝试存款
    test.asset_token.approve(&test.admin, &test.contract.address, &deposit_amount, &99999);
    test.contract.deposit(&deposit_amount);
}

#[test]
#[should_panic]
fn test_withdraw_without_pending_amount() {
    let test = PumpBTCStakingTest::setup_initialized();
    
    // 设置操作员
    test.contract.set_operator(&test.operator);
    
    // 直接尝试提取，没有待质押金额，应该失败
    test.contract.withdraw();
}

#[test]
#[should_panic]
fn test_collect_fee_without_fee() {
    let test = PumpBTCStakingTest::setup_initialized();
    
    // 直接尝试收取手续费，没有手续费可收，应该失败
    test.contract.collect_fee();
}

#[test]
#[should_panic]
fn test_set_invalid_fee() {
    let test = PumpBTCStakingTest::setup_initialized();
    
    // 尝试设置超过100%的手续费，应该失败
    test.contract.set_instant_unstake_fee(&10001); // 超过10000 (100%)
}

#[test]
#[should_panic]
fn test_claim_slot_too_early() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 100_000_000i128;
    let unstake_amount = 50_000_000i128;
    
    // 执行质押和解质押请求
    test.contract.set_stake_asset_cap(&(10_000_000_000i128));
    test.contract.set_operator(&test.operator);
    
    let total_deposit = stake_amount + unstake_amount;
    test.asset_token.approve(&test.operator, &test.contract.address, &total_deposit, &99999);
    test.contract.deposit(&total_deposit);
    
    test.pump_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);
    test.contract.stake(&test.user1, &stake_amount);
    
    test.contract.set_only_allow_stake(&false);
    test.contract.unstake_request(&test.user1, &unstake_amount);
    
    // 获取槽位信息
    let slot = test.contract.get_max_date_slot();
    
    // 不等待足够的时间，直接尝试领取，应该失败
    test.contract.claim_slot(&test.user1, &slot);
}

#[test]
#[should_panic]
fn test_pause_when_already_paused() {
    let test = PumpBTCStakingTest::setup_initialized();
    
    // 暂停合约
    test.contract.pause();
    
    // 再次尝试暂停，应该失败
    test.contract.pause();
}

#[test]
#[should_panic]
fn test_unpause_when_not_paused() {
    let test = PumpBTCStakingTest::setup_initialized();
    
    // 合约默认未暂停，尝试恢复，应该失败
    test.contract.unpause();
}

// ========================= 复杂场景测试 =========================
#[test]
fn test_multiple_users_stake_and_unstake() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 100_000_000i128;
    
    // 设置质押上限
    test.contract.set_stake_asset_cap(&(10_000_000_000i128));
    test.contract.set_operator(&test.operator);
    
    // 存入足够的资产
    let total_deposit = stake_amount * 2;
    test.asset_token.approve(&test.operator, &test.contract.address, &total_deposit, &99999);
    test.contract.deposit(&total_deposit);
    
    // 用户1和用户2都进行质押
    test.pump_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);
    test.pump_token.approve(&test.user2, &test.contract.address, &stake_amount, &99999);
    
    test.contract.stake(&test.user1, &stake_amount);
    test.contract.stake(&test.user2, &stake_amount);
    
    // 验证总质押金额
    assert_eq!(stake_amount * 2, test.contract.get_total_staking_amount());
    
    // 允许解质押
    test.contract.set_only_allow_stake(&false);
    
    // 用户1请求解质押一半
    let unstake_amount = 50_000_000i128;
    test.contract.unstake_request(&test.user1, &unstake_amount);
    
    // 验证总请求解质押金额
    assert_eq!(unstake_amount, test.contract.get_total_requested_amount());
    
    // 验证总质押金额减少
    assert_eq!(stake_amount * 2 - unstake_amount, test.contract.get_total_staking_amount());
}

#[test]
fn test_claim_all() {
    let test = PumpBTCStakingTest::setup_initialized();
    let stake_amount = 100_000_000i128;
    let unstake_amount1 = 30_000_000i128;
    let unstake_amount2 = 20_000_000i128;
    
    // 执行质押
    test.contract.set_stake_asset_cap(&(10_000_000_000i128));
    test.contract.set_operator(&test.operator);
    
    let total_deposit = stake_amount + unstake_amount1 + unstake_amount2;
    test.asset_token.approve(&test.operator, &test.contract.address, &total_deposit, &99999);
    test.contract.deposit(&total_deposit);
    
    test.pump_token.approve(&test.user1, &test.contract.address, &stake_amount, &99999);
    test.contract.stake(&test.user1, &stake_amount);
    
    // 允许解质押
    test.contract.set_only_allow_stake(&false);
    
    // 进行两次解质押请求（在不同时间）
    test.contract.unstake_request(&test.user1, &unstake_amount1);
    
    // 模拟时间推进
    test.env.ledger().with_mut(|ledger| {
        ledger.timestamp = ledger.timestamp + 1 * 24 * 60 * 60; // 1天后
    });
    
    test.contract.unstake_request(&test.user1, &unstake_amount2);
    
    // 模拟时间推进到可以领取
    test.env.ledger().with_mut(|ledger| {
        ledger.timestamp = ledger.timestamp + 7 * 24 * 60 * 60; // 再7天后
    });
    
    // 操作员处理解质押请求
    let remaining_deposit = total_deposit - unstake_amount1 - unstake_amount2;
    test.asset_token.approve(&test.operator, &test.contract.address, &remaining_deposit, &99999);
    test.contract.withdraw_and_deposit(&remaining_deposit);
    
    // 用户一次性领取所有可领取的解质押资产
    test.contract.claim_all(&test.user1);
    
    // 验证总请求解质押金额清零
    assert_eq!(0i128, test.contract.get_total_requested_amount());
}