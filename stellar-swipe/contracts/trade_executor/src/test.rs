#![cfg(test)]
//! Additional integration tests; stop-loss / take-profit coverage is in `triggers::tests`.

use crate::TradeExecutorContract;
use soroban_sdk::Env;

#[test]
fn register_contract() {
    let env = Env::default();
    let _id = env.register(TradeExecutorContract, ());
}
