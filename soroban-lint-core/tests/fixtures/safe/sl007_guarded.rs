#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct C;

#[contractimpl]
impl C {
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&INIT) {
            panic_with_error!(env, Error);
        }
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&INIT, &true);
    }
}
