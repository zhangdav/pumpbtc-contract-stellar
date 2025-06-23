use crate::tests::test_setup::PumpBTCStakingTest;

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