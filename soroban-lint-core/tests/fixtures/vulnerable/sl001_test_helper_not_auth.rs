#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn set_balance(env: Env, addr: Address, amount: i128) {
        Self::check(&addr);
        env.storage().persistent().set(&addr, &amount);
    }

    #[cfg(test)]
    fn check(addr: &Address) {
        addr.require_auth();
    }
}
