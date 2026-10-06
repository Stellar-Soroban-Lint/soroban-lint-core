#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct C;

#[contractimpl]
impl C {
    pub fn init(env: Env, admin: Address) {
        env.storage().instance().set(&ADMIN, &admin);
    }
}
