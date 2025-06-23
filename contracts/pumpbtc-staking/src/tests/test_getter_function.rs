use crate::tests::test_setup::PumpBTCStakingTest;

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