use soroban_sdk::{contracttype, Address, Env, symbol_short};


#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferAdminEvent {
    pub previous_admin: Address,
    pub new_admin: Address,
}

pub(crate) fn transfer_admin(e: &Env, previous_admin: Address, new_admin: Address) {
    let event: TransferAdminEvent = TransferAdminEvent {
        previous_admin,
        new_admin,
    };
    e.events()
        .publish(("PumpToken", symbol_short!("tra_admin")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptAdminEvent {
    pub previous_admin: Address,
    pub new_admin: Address,
}

pub(crate) fn accept_admin(e: &Env, previous_admin: Address, new_admin: Address) {
    let event: AcceptAdminEvent = AcceptAdminEvent {
        previous_admin,
        new_admin,
    };
    e.events()
        .publish(("PumpToken", symbol_short!("acc_admin")), event);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenounceAdminEvent {
    pub admin: Address,
}

pub(crate) fn renounce_admin(e: &Env, admin: Address) {
    let event: RenounceAdminEvent = RenounceAdminEvent { admin };
    e.events()
        .publish(("PumpToken", symbol_short!("ren_admin")), event);
}
