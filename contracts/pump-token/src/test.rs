#![cfg(test)]
extern crate std;

use crate::{contract::PumpToken, PumpTokenClient};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol,
};

fn create_token<'a>(e: &Env, admin: &Address, minter: &Address) -> PumpTokenClient<'a> {
    let token = PumpTokenClient::new(e, &e.register(PumpToken, ()));
    token.initialize(
        admin,
        minter,
        &"pumpBTC".into_val(e),
        &"pumpBTC".into_val(e),
    );
    token
}

#[test]
fn test() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let minter1 = Address::generate(&e);
    let minter2 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);
    let token = create_token(&e, &admin1, &minter1);

    // Test set new minter
    token.set_minter(&minter2);

    token.mint(&user1, &1000);
    assert_eq!(
        e.auths(),
        std::vec![(
            minter2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&user1, 1000_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user2, &user3, &500, &200);
    assert_eq!(
        e.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("approve"),
                    (&user2, &user3, 500_i128, 200_u32).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.allowance(&user2, &user3), 500);

    token.transfer(&user1, &user2, &600);
    assert_eq!(
        e.auths(),
        std::vec![(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("transfer"),
                    (&user1, &user2, 600_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 400);
    assert_eq!(token.balance(&user2), 600);

    token.transfer_from(&user3, &user2, &user1, &400);
    assert_eq!(
        e.auths(),
        std::vec![(
            user3.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    Symbol::new(&e, "transfer_from"),
                    (&user3, &user2, &user1, 400_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 800);
    assert_eq!(token.balance(&user2), 200);

    token.transfer(&user1, &user3, &300);
    assert_eq!(token.balance(&user1), 500);
    assert_eq!(token.balance(&user3), 300);

    // Increase to 500
    token.approve(&user2, &user3, &500, &200);
    assert_eq!(token.allowance(&user2, &user3), 500);
    token.approve(&user2, &user3, &0, &200);
    assert_eq!(
        e.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("approve"),
                    (&user2, &user3, 0_i128, 200_u32).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.allowance(&user2, &user3), 0);
}

#[test]
fn test_burn() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user1, &user2, &500, &200);
    assert_eq!(token.allowance(&user1, &user2), 500);

    token.burn_from(&user2, &user1, &500);
    assert_eq!(
        e.auths(),
        std::vec![(
            minter.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn_from"),
                    (&user2, &user1, 500_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.allowance(&user1, &user2), 0);
    assert_eq!(token.balance(&user1), 500);
    assert_eq!(token.balance(&user2), 0);

    token.burn(&user1, &500);
    assert_eq!(
        e.auths(),
        std::vec![(
            minter.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn"),
                    (&user1, 500_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&user2), 0);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn transfer_insufficient_balance() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.transfer(&user1, &user2, &1001);
}

#[test]
#[should_panic(expected = "insufficient allowance")]
fn transfer_from_insufficient_allowance() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user1, &user3, &100, &200);
    assert_eq!(token.allowance(&user1, &user3), 100);

    token.transfer_from(&user3, &user1, &user2, &101);
}

#[test]
#[should_panic(expected = "already initialized")]
fn initialize_already_initialized() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.initialize(
        &admin,
        &minter,
        &"name".into_val(&e),
        &"symbol".into_val(&e),
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_non_minter_cannot_mint() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user = Address::generate(&e);

    let token = create_token(&e, &admin, &minter);

    token.mint(&user, &1000);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_non_minter_cannot_burn() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user = Address::generate(&e);

    let token = create_token(&e, &admin, &minter);
    token.mint(&user, &1000);

    token.burn(&user, &500);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_non_minter_cannot_burn_from() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    let token = create_token(&e, &admin, &minter);
    token.mint(&user1, &1000);
    token.approve(&user1, &user2, &500, &200);

    token.burn_from(&user2, &user1, &500);
}

#[test]
fn test_zero_allowance() {
    // Here we test that transfer_from with a 0 amount does not create an empty allowance
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let spender = Address::generate(&e);
    let from = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.transfer_from(&spender, &from, &spender, &0);
    assert!(token.get_allowance(&from, &spender).is_none());
}

#[test]
fn test_metadata() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    assert_eq!(token.name(), "pumpBTC".into_val(&e));
    assert_eq!(token.symbol(), "pumpBTC".into_val(&e));
    assert_eq!(token.decimals(), 8);
}

#[test]
fn test_get_minter() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter1 = Address::generate(&e);
    let minter2 = Address::generate(&e);
    let token = create_token(&e, &admin, &minter1);

    // Test initial minter
    assert_eq!(token.get_minter(), minter1);

    // Test after setting new minter
    token.set_minter(&minter2);
    assert_eq!(token.get_minter(), minter2);
}

#[test]
fn test_admin_transfer() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let admin2 = Address::generate(&e);
    let minter = Address::generate(&e);
    let token = create_token(&e, &admin1, &minter);

    // Initially no pending admin
    assert!(token.get_pending_admin().is_none());

    // Transfer admin to admin2
    token.transfer_admin(&admin2);
    assert_eq!(
        e.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    Symbol::new(&e, "transfer_admin"),
                    (&admin2,).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // Check pending admin is set
    assert_eq!(token.get_pending_admin(), Some(admin2.clone()));

    // Accept admin transfer
    token.accept_admin();
    assert_eq!(
        e.auths(),
        std::vec![(
            admin2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    Symbol::new(&e, "accept_admin"),
                    ().into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // Check pending admin is cleared
    assert!(token.get_pending_admin().is_none());

    // Verify new admin can set minter
    let new_minter = Address::generate(&e);
    token.set_minter(&new_minter);
    assert_eq!(token.get_minter(), new_minter);
}

#[test]
fn test_renounce_admin() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let admin2 = Address::generate(&e);
    let minter = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    // Set pending admin first
    token.transfer_admin(&admin2);
    assert_eq!(token.get_pending_admin(), Some(admin2.clone()));

    // Renounce admin - should clear pending admin
    token.renounce_admin();
    assert_eq!(
        e.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    Symbol::new(&e, "renounce_admin"),
                    ().into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // Check pending admin is cleared
    assert!(token.get_pending_admin().is_none());
}

#[test]
#[should_panic(expected = "Error(Contract, #0)")]
fn test_accept_admin_no_pending() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    // Try to accept admin when no pending transfer
    token.accept_admin();
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_transfer_admin_unauthorized() {
    let e = Env::default();

    let admin = Address::generate(&e);
    let new_admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    // Try to transfer admin without proper authorization
    token.transfer_admin(&new_admin);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_minter_unauthorized() {
    let e = Env::default();

    let admin = Address::generate(&e);
    let new_minter = Address::generate(&e);
    let minter = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    // Try to set minter without proper authorization
    token.set_minter(&new_minter);
}

#[test]
#[should_panic(expected = "negative amount is not allowed")]
fn test_mint_negative_amount() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.mint(&user, &-100);
}

#[test]
#[should_panic(expected = "negative amount is not allowed")]
fn test_transfer_negative_amount() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.mint(&user1, &1000);
    token.transfer(&user1, &user2, &-100);
}

#[test]
#[should_panic(expected = "negative amount is not allowed")]
fn test_approve_negative_amount() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.approve(&user1, &user2, &-100, &200);
}

#[test]
#[should_panic(expected = "negative amount is not allowed")]
fn test_burn_negative_amount() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.mint(&user, &1000);
    token.burn(&user, &-100);
}

#[test]
#[should_panic(expected = "negative amount is not allowed")]
fn test_transfer_from_negative_amount() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.mint(&user1, &1000);
    token.approve(&user1, &user3, &500, &200);
    token.transfer_from(&user3, &user1, &user2, &-100);
}

#[test]
#[should_panic(expected = "negative amount is not allowed")]
fn test_burn_from_negative_amount() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    token.mint(&user1, &1000);
    token.approve(&user1, &user2, &500, &200);
    token.burn_from(&user2, &user1, &-100);
}

#[test]
fn test_get_allowance_with_data() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let minter = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let token = create_token(&e, &admin, &minter);

    // Initially no allowance
    assert!(token.get_allowance(&user1, &user2).is_none());

    // Set allowance
    token.approve(&user1, &user2, &500, &200);

    // Check allowance exists
    let allowance = token.get_allowance(&user1, &user2);
    assert!(allowance.is_some());
    let allowance_value = allowance.unwrap();
    assert_eq!(allowance_value.amount, 500);
    assert_eq!(allowance_value.expiration_ledger, 200);
}

#[test]
fn test_complex_admin_transfer_scenario() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let admin2 = Address::generate(&e);
    let admin3 = Address::generate(&e);
    let minter = Address::generate(&e);
    let token = create_token(&e, &admin1, &minter);

    // Transfer admin1 -> admin2
    token.transfer_admin(&admin2);
    assert_eq!(token.get_pending_admin(), Some(admin2.clone()));

    // Before accepting, transfer to admin3 (should override pending)
    token.transfer_admin(&admin3);
    assert_eq!(token.get_pending_admin(), Some(admin3.clone()));

    // Accept as admin3
    token.accept_admin();
    assert!(token.get_pending_admin().is_none());

    // Verify admin3 is now the admin by setting minter
    let new_minter = Address::generate(&e);
    token.set_minter(&new_minter);
    assert_eq!(token.get_minter(), new_minter);
}
