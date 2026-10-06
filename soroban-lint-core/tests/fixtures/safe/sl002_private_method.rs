#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn get(env: Env, key: u32) -> u32 {
        Self::read(&env, key)
    }

    fn read(env: &Env, key: u32) -> u32 {
        env.storage().instance().get(&key).unwrap()
    }
}
