use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct C;

#[contractimpl]
impl C {
    pub fn read(_env: Env) -> u32 {
        unsafe { 42 }
    }
}
