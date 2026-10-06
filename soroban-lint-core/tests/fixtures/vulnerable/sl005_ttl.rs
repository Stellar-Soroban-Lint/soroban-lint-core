#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct C;

#[contractimpl]
impl C {
    pub fn save(env: Env, v: u32) {
        env.storage().persistent().set(&KEY, &v);
    }
}
