#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

fn check(addr: &Address) {
    addr.require_auth();
}

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn set_balance(env: Env, addr: Address, amount: i128) {
        check(&addr);
        env.storage().persistent().set(&addr, &amount);
    }
}
