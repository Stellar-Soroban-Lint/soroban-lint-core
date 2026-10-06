#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct C;

#[contractimpl]
impl C {
    pub fn get(env: Env, key: u32) -> u32 {
        let v: u32 = env.storage().instance().get(&key).unwrap();
        v
    }
}
