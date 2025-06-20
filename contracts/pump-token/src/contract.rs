use crate::admin::{has_administrator, read_administrator, write_administrator, read_pending_administrator, remove_pending_administrator, write_pending_administrator};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::metadata::{read_decimal, read_name, read_symbol, write_metadata, DECIMAL};
use crate::minter::{read_minter, write_minter};
use crate::error::PumpTokenError;
use crate::storage_types::{AllowanceDataKey, AllowanceValue, DataKey};
use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use soroban_sdk::token::{self, Interface as _};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;
use crate::event;

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

/// PumpToken - Custom token contract with Ownable2Step and minting capabilities
pub trait PumpTokenTrait {
    fn initialize(
        e: Env,
        admin: Address,
        minter: Address,
        name: String,
        symbol: String,
    );
    
    fn transfer_admin(e: Env, new_admin: Address) -> Result<(), PumpTokenError>;
    fn accept_admin(e: Env) -> Result<(), PumpTokenError>;
    fn renounce_admin(e: Env) -> Result<(), PumpTokenError>;
    fn get_pending_admin(e: Env) -> Option<Address>;
    
    fn mint(e: Env, to: Address, amount: i128);
    fn set_minter(e: Env, new_minter: Address);
    fn get_minter(e: Env) -> Address;
    
    fn get_allowance(e: Env, from: Address, spender: Address) -> Option<AllowanceValue>;
}

#[contract]
pub struct PumpToken;

#[contractimpl]
impl PumpTokenTrait for PumpToken {
    fn initialize(
        e: Env,
        admin: Address,
        minter: Address,
        name: String,
        symbol: String,
    ) {
        if has_administrator(&e) {
            panic!("already initialized")
        }
        write_administrator(&e, &admin);

        write_minter(&e, &minter);

        write_metadata(
            &e,
            TokenMetadata {
                decimal: DECIMAL,
                name,
                symbol,
            },
        )
    }

    fn transfer_admin(e: Env, new_admin: Address) -> Result<(), PumpTokenError> {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let admin = read_administrator(&e);
        admin.require_auth();

        write_pending_administrator(&e, &new_admin);
        event::transfer_admin(&e, admin, new_admin);

        Ok(())
    }

    fn accept_admin(e: Env) -> Result<(), PumpTokenError> {
        e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let pending_admin = read_pending_administrator(&e);
        if pending_admin.is_none() {
            return Err(PumpTokenError::NoPendingAdminTransfer);
        }

        let pending_admin = pending_admin.unwrap();
        pending_admin.require_auth();

        let old_admin = read_administrator(&e);
        write_administrator(&e, &pending_admin);
        remove_pending_administrator(&e);

        event::accept_admin(&e, old_admin, pending_admin);
        Ok(())
    }

    fn renounce_admin(e: Env) -> Result<(), PumpTokenError> {
        e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let admin = read_administrator(&e);
        admin.require_auth();

        remove_pending_administrator(&e);
        event::renounce_admin(&e, admin);

        Ok(())
    }

    fn get_pending_admin(e: Env) -> Option<Address> {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_pending_administrator(&e)
    }

    fn mint(e: Env, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        let minter = read_minter(&e);
        minter.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().mint(minter, to, amount);
    }

    fn set_minter(e: Env, new_minter: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_minter(&e, &new_minter);
        e.events()
            .publish((symbol_short!("minter"), admin), new_minter);
    }

    fn get_minter(e: Env) -> Address {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_minter(&e)
    }

    fn get_allowance(e: Env, from: Address, spender: Address) -> Option<AllowanceValue> {
        let key = DataKey::Allowance(AllowanceDataKey { from, spender });
        let allowance = e.storage().temporary().get::<_, AllowanceValue>(&key);
        allowance
    }
}

#[contractimpl]
impl token::Interface for PumpToken {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_allowance(&e, from, spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        TokenUtils::new(&e)
            .events()
            .approve(from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_balance(&e, id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount)
    }

    // Only minter can call burn
    fn burn(e: Env, from: Address, amount: i128) {
        let minter = read_minter(&e);
        minter.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount);
    }

    // Only minter can call burn_from
    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        let minter = read_minter(&e);
        minter.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount)
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> String {
        read_name(&e)
    }

    fn symbol(e: Env) -> String {
        read_symbol(&e)
    }
}
