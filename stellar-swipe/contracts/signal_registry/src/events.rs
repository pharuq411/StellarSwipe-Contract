use crate::types::Asset;
use soroban_sdk::{Address, Env, Symbol};

pub fn emit_admin_transferred(env: &Env, old_admin: Address, new_admin: Address) {
    let topics = (Symbol::new(env, "admin_transferred"), old_admin, new_admin);
    env.events().publish(topics, ());
}

pub fn emit_parameter_updated(env: &Env, parameter: Symbol, old_value: i128, new_value: i128) {
    let topics = (Symbol::new(env, "parameter_updated"), parameter);
    env.events().publish(topics, (old_value, new_value));
}

pub fn emit_trading_paused(env: &Env, paused_by: Address, expires_at: u64) {
    let topics = (Symbol::new(env, "trading_paused"), paused_by);
    let timestamp = env.ledger().timestamp();
    env.events().publish(topics, (timestamp, expires_at));
}

pub fn emit_trading_unpaused(env: &Env, unpaused_by: Address) {
    let topics = (Symbol::new(env, "trading_unpaused"), unpaused_by);
    let timestamp = env.ledger().timestamp();
    env.events().publish(topics, timestamp);
}

pub fn emit_multisig_signer_added(env: &Env, signer: Address, added_by: Address) {
    let topics = (Symbol::new(env, "multisig_signer_added"), signer, added_by);
    env.events().publish(topics, ());
}

pub fn emit_multisig_signer_removed(env: &Env, signer: Address, removed_by: Address) {
    let topics = (
        Symbol::new(env, "multisig_signer_removed"),
        signer,
        removed_by,
    );
    env.events().publish(topics, ());
}

pub fn emit_fee_collected(
    env: &Env,
    asset: Asset,
    total_fee: i128,
    platform_fee: i128,
    provider_fee: i128,
    provider: Address,
    platform_treasury: Address,
) {
    let topics = (
        Symbol::new(env, "fee_collected"),
        asset.symbol,
        provider,
        platform_treasury,
    );
    env.events()
        .publish(topics, (total_fee, platform_fee, provider_fee));
}

pub fn emit_signal_expired(env: &Env, signal_id: u64, provider: Address, expiry_time: u64) {
    let topics = (Symbol::new(env, "signal_expired"), provider, signal_id);
    env.events().publish(topics, expiry_time);
}

pub fn emit_trade_executed(env: &Env, signal_id: u64, executor: Address, roi: i128, volume: i128) {
    let topics = (Symbol::new(env, "trade_executed"), signal_id, executor);
    env.events().publish(topics, (roi, volume));
}

pub fn emit_signal_status_changed(
    env: &Env,
    signal_id: u64,
    provider: Address,
    old_status: u32,
    new_status: u32,
) {
    let topics = (
        Symbol::new(env, "signal_status_changed"),
        signal_id,
        provider,
    );
    env.events().publish(topics, (old_status, new_status));
}

pub fn emit_provider_stats_updated(
    env: &Env,
    provider: Address,
    success_rate: u32,
    avg_return: i128,
    total_volume: i128,
) {
    let topics = (Symbol::new(env, "provider_stats_updated"), provider);
    env.events()
        .publish(topics, (success_rate, avg_return, total_volume));
}

pub fn emit_verification_eligibility_checked(env: &Env, provider: Address, eligible: bool) {
    let topics = (
        Symbol::new(env, "verification_eligibility_checked"),
        provider,
    );
    env.events().publish(topics, eligible);
}

pub fn emit_follow_gained(env: &Env, user: Address, provider: Address, new_count: u32) {
    let topics = (Symbol::new(env, "follow_gained"), provider, user);
    env.events().publish(topics, new_count);
}

pub fn emit_follow_lost(env: &Env, user: Address, provider: Address, new_count: u32) {
    let topics = (Symbol::new(env, "follow_lost"), provider, user);
    env.events().publish(topics, new_count);
}
