use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, IntoVal,
};

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
pub mod pumpbtc_staking {
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
