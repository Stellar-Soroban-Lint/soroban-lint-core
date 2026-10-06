#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct C;

#[contractimpl]
impl C {
    pub fn cache(env: Env, user: Address, balance: i128) {
        env.storage().temporary().set(&user, &balance);
    }
}
