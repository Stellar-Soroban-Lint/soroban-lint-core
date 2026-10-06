#![no_std]
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct C;

#[contractimpl]
impl C {
    pub fn add(a: u64, b: u64) -> u64 {
        a + b
    }
}
