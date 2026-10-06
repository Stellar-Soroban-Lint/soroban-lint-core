#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct C;

#[contractimpl]
impl C {
    pub fn add_item(env: Env, item: u32) {
        let mut list: soroban_sdk::Vec<u32> =
            env.storage().persistent().get(&KEY).unwrap_or_default();
        list.push_back(item);
        env.storage().persistent().set(&KEY, &list);
    }
}
