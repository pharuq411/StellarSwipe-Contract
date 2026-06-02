use soroban_sdk::{contracttype, Address, Env, String};
use stellar_swipe_common::Asset;
use shared::errors::{ErrorCategory, RecoveryStrategy};

pub const MAX_FEE_RATE_BPS: u32 = 100; // 1%
pub const MIN_FEE_RATE_BPS: u32 = 1; // 0.01%
pub const DEFAULT_FEE_RATE_BPS: u32 = 30; // 0.3%
pub const DEFAULT_BURN_RATE_BPS: u32 = 1_000; // 10%
pub const MAX_BURN_RATE_BPS: u32 = 10_000; // 100%
pub const DEFAULT_NETWORK_SCORE_BPS: u32 = 0;
pub const DEFAULT_FEE_OPTIMIZATION_MAX_RATE_BPS: u32 = 100;
pub const DEFAULT_CONGESTION_SENSITIVITY_BPS: u32 = 50;
pub const DEFAULT_MAX_RETRY_ATTEMPTS: u32 = 3;
pub const LEDGERS_PER_MONTH_APPROX: u32 = 518_400; // ~30 days at ~5 seconds per ledger
pub const SILVER_TIER_VOLUME_USD: i128 = 10_000 * 10_000_000; // $10k, 7 decimals
pub const GOLD_TIER_VOLUME_USD: i128 = 50_000 * 10_000_000; // $50k, 7 decimals
pub const SILVER_DISCOUNT_BPS: u32 = 5;
pub const GOLD_DISCOUNT_BPS: u32 = 10;

#[contracttype]
pub enum StorageKey {
    Admin,
    Initialized,
    OracleContract,
    TreasuryBalance(Address),              // persistent, per-token
    QueuedWithdrawal,                      // instance, single-slot
    FeeRate,                               // instance, current fee rate in bps
    BurnRate,                              // instance, burn rate in bps
    ProviderPendingFees(Address, Address), // persistent, per (provider, token)
    MonthlyTradeVolume(Address),           // persistent, per user
    /// Accumulated fee shares per provider per day (day = unix_timestamp / SECONDS_PER_DAY).
    ProviderDailyFeeShares(Address, u64),
    /// Day number of the provider's first recorded earnings (for ALL_TIME period_start).
    ProviderEarningsFirstDay(Address),
    /// Total accumulated fee shares for a provider, used to rank earnings leaders.
    ProviderTotalEarnings(Address),
    /// Providers that have recorded earnings, for leaderboard scans.
    ProviderEarningsIndex,
    /// Whether a user has completed their first trade (Issue #428).
    HasTraded(Address),
    // ── Issue #438: Protocol Token Integration ─────────────────────
    /// Optional protocol token address for token-based fee payment.
    ProtocolToken,
    /// Revenue share rate in basis points (default: 2000 = 20%).
    RevenueShareRateBps,
    /// Last snapshot ledger for revenue sharing (Issue #442).
    LastRevenueShareSnapshot,
    /// Accumulated revenue share pool waiting for next distribution.
    RevenueSharePool(Address),
    /// Latest aggregated network score for fee optimization.
    NetworkConditionScore,
    /// Configurable dynamic fee optimization parameters.
    FeeOptimizationConfig,
    /// Last recorded contract error report.
    LastErrorReport,
    /// Persisted failed fee collection operation for retry.
    FailedFeeCollection(String),
}

#[contracttype]
#[derive(Clone)]
pub struct FeeOptimizationConfig {
    pub max_dynamic_rate_bps: u32,
    pub congestion_sensitivity_bps: u32,
    pub min_effective_rate_bps: u32,
    pub max_retry_attempts: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct ErrorReport {
    pub category: ErrorCategory,
    pub strategy: RecoveryStrategy,
    pub message: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct FailedFeeCollection {
    pub id: String,
    pub trader: Address,
    pub token: Address,
    pub trade_amount: i128,
    pub trade_asset: Asset,
    pub retry_count: u32,
    pub last_error: String,
}

#[contracttype]
#[derive(Clone)]
pub struct QueuedWithdrawal {
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub queued_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct MonthlyTradeVolume {
    pub month_bucket: u32,
    pub volume_usd: i128,
}

// --- Admin ---

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&StorageKey::Admin).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&StorageKey::Admin, admin);
}

// --- Initialized ---

pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&StorageKey::Initialized)
        .unwrap_or(false)
}

pub fn set_initialized(env: &Env) {
    env.storage()
        .instance()
        .set(&StorageKey::Initialized, &true);
}

// --- Oracle Contract ---

pub fn get_oracle_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&StorageKey::OracleContract)
}

pub fn set_oracle_contract(env: &Env, contract: &Address) {
    env.storage()
        .instance()
        .set(&StorageKey::OracleContract, contract);
}

// --- Treasury Balance ---

pub fn get_treasury_balance(env: &Env, token: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&StorageKey::TreasuryBalance(token.clone()))
        .unwrap_or(0i128)
}

pub fn set_treasury_balance(env: &Env, token: &Address, balance: i128) {
    env.storage()
        .persistent()
        .set(&StorageKey::TreasuryBalance(token.clone()), &balance);
}

// --- Queued Withdrawal ---

pub fn get_queued_withdrawal(env: &Env) -> Option<QueuedWithdrawal> {
    env.storage().instance().get(&StorageKey::QueuedWithdrawal)
}

pub fn set_queued_withdrawal(env: &Env, withdrawal: &QueuedWithdrawal) {
    env.storage()
        .instance()
        .set(&StorageKey::QueuedWithdrawal, withdrawal);
}

pub fn remove_queued_withdrawal(env: &Env) {
    env.storage()
        .instance()
        .remove(&StorageKey::QueuedWithdrawal);
}

// --- Fee Rate ---

pub fn get_fee_rate(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&StorageKey::FeeRate)
        .unwrap_or(DEFAULT_FEE_RATE_BPS)
}

pub fn set_fee_rate(env: &Env, rate: u32) {
    env.storage().instance().set(&StorageKey::FeeRate, &rate);
}

// --- Fee Optimization ---

pub fn get_network_condition_score(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&StorageKey::NetworkConditionScore)
        .unwrap_or(DEFAULT_NETWORK_SCORE_BPS)
}

pub fn set_network_condition_score(env: &Env, score: u32) {
    env.storage()
        .instance()
        .set(&StorageKey::NetworkConditionScore, &score);
}

pub fn get_fee_optimization_config(env: &Env) -> FeeOptimizationConfig {
    env.storage()
        .instance()
        .get(&StorageKey::FeeOptimizationConfig)
        .unwrap_or(FeeOptimizationConfig {
            max_dynamic_rate_bps: DEFAULT_FEE_OPTIMIZATION_MAX_RATE_BPS,
            congestion_sensitivity_bps: DEFAULT_CONGESTION_SENSITIVITY_BPS,
            min_effective_rate_bps: MIN_FEE_RATE_BPS,
            max_retry_attempts: DEFAULT_MAX_RETRY_ATTEMPTS,
        })
}

pub fn set_fee_optimization_config(env: &Env, config: &FeeOptimizationConfig) {
    env.storage()
        .instance()
        .set(&StorageKey::FeeOptimizationConfig, config);
}

pub fn get_last_error_report(env: &Env) -> Option<ErrorReport> {
    env.storage().instance().get(&StorageKey::LastErrorReport)
}

pub fn set_last_error_report(env: &Env, report: &ErrorReport) {
    env.storage()
        .instance()
        .set(&StorageKey::LastErrorReport, report);
}

pub fn get_failed_fee_collection(env: &Env, id: &String) -> Option<FailedFeeCollection> {
    env.storage()
        .persistent()
        .get(&StorageKey::FailedFeeCollection(id.clone()))
}

pub fn set_failed_fee_collection(env: &Env, failed: &FailedFeeCollection) {
    env.storage()
        .persistent()
        .set(&StorageKey::FailedFeeCollection(failed.id.clone()), failed);
}

pub fn remove_failed_fee_collection(env: &Env, id: &String) {
    env.storage()
        .persistent()
        .remove(&StorageKey::FailedFeeCollection(id.clone()));
}

// --- Burn Rate ---

pub fn get_burn_rate(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&StorageKey::BurnRate)
        .unwrap_or(DEFAULT_BURN_RATE_BPS)
}

pub fn set_burn_rate(env: &Env, rate: u32) {
    env.storage().instance().set(&StorageKey::BurnRate, &rate);
}

// --- Provider Pending Fees ---

pub fn get_pending_fees(env: &Env, provider: &Address, token: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&StorageKey::ProviderPendingFees(
            provider.clone(),
            token.clone(),
        ))
        .unwrap_or(0i128)
}

pub fn set_pending_fees(env: &Env, provider: &Address, token: &Address, amount: i128) {
    env.storage().persistent().set(
        &StorageKey::ProviderPendingFees(provider.clone(), token.clone()),
        &amount,
    );
}

// --- Monthly Trade Volume ---

pub fn get_monthly_trade_volume(env: &Env, user: &Address) -> Option<MonthlyTradeVolume> {
    env.storage()
        .persistent()
        .get(&StorageKey::MonthlyTradeVolume(user.clone()))
}

pub fn set_monthly_trade_volume(env: &Env, user: &Address, volume: &MonthlyTradeVolume) {
    env.storage()
        .persistent()
        .set(&StorageKey::MonthlyTradeVolume(user.clone()), volume);
}

pub fn remove_monthly_trade_volume(env: &Env, user: &Address) {
    env.storage()
        .persistent()
        .remove(&StorageKey::MonthlyTradeVolume(user.clone()));
}

// --- Provider Daily Fee Shares (Issue #366) ---

pub fn get_provider_daily_fee_shares(env: &Env, provider: &Address, day: u64) -> i128 {
    env.storage()
        .persistent()
        .get(&StorageKey::ProviderDailyFeeShares(provider.clone(), day))
        .unwrap_or(0i128)
}

pub fn get_provider_total_earnings(env: &Env, provider: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&StorageKey::ProviderTotalEarnings(provider.clone()))
        .unwrap_or(0i128)
}

pub fn get_provider_earnings_index(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&StorageKey::ProviderEarningsIndex)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_provider_to_earnings_index(env: &Env, provider: &Address) {
    let mut index = get_provider_earnings_index(env);
    for i in 0..index.len() {
        if index.get(i).unwrap() == *provider {
            return;
        }
    }
    index.push_back(provider.clone());
    env.storage()
        .persistent()
        .set(&StorageKey::ProviderEarningsIndex, &index);
}

pub fn add_provider_total_earnings(env: &Env, provider: &Address, amount: i128) {
    let key = StorageKey::ProviderTotalEarnings(provider.clone());
    let current: i128 = env.storage().persistent().get(&key).unwrap_or(0i128);
    let updated = current.saturating_add(amount);
    env.storage().persistent().set(&key, &updated);
    add_provider_to_earnings_index(env, provider);
}

pub fn add_provider_daily_fee_shares(env: &Env, provider: &Address, day: u64, amount: i128) {
    let key = StorageKey::ProviderDailyFeeShares(provider.clone(), day);
    let current: i128 = env.storage().persistent().get(&key).unwrap_or(0i128);
    let updated = current.saturating_add(amount);
    env.storage().persistent().set(&key, &updated);

    // Record first earnings day if not yet set
    let first_key = StorageKey::ProviderEarningsFirstDay(provider.clone());
    if !env.storage().persistent().has(&first_key) {
        env.storage().persistent().set(&first_key, &day);
    }
    add_provider_total_earnings(env, provider, amount);
}

pub fn get_provider_earnings_first_day(env: &Env, provider: &Address) -> Option<u64> {
    env.storage()
        .persistent()
        .get(&StorageKey::ProviderEarningsFirstDay(provider.clone()))
}

// --- First-trade tracking (Issue #428) ---

pub fn has_traded(env: &Env, user: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&StorageKey::HasTraded(user.clone()))
        .unwrap_or(false)
}

pub fn set_has_traded(env: &Env, user: &Address) {
    env.storage()
        .persistent()
        .set(&StorageKey::HasTraded(user.clone()), &true);
}

// ── Issue #438: Protocol Token ──────────────────────────────────────

pub fn get_protocol_token(env: &Env) -> Option<Address> {
    env.storage().instance().get(&StorageKey::ProtocolToken)
}

pub fn set_protocol_token(env: &Env, token: &Address) {
    env.storage()
        .instance()
        .set(&StorageKey::ProtocolToken, token);
}

// ── Issue #442: Revenue Share ────────────────────────────────────────

pub const DEFAULT_REVENUE_SHARE_RATE_BPS: u32 = 2000; // 20%
pub const SECONDS_PER_WEEK: u64 = 604_800;

pub fn get_revenue_share_rate_bps(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&StorageKey::RevenueShareRateBps)
        .unwrap_or(DEFAULT_REVENUE_SHARE_RATE_BPS)
}

pub fn set_revenue_share_rate_bps(env: &Env, rate_bps: u32) {
    env.storage()
        .instance()
        .set(&StorageKey::RevenueShareRateBps, &rate_bps);
}

pub fn get_last_revenue_share_snapshot(env: &Env) -> Option<u64> {
    env.storage()
        .instance()
        .get(&StorageKey::LastRevenueShareSnapshot)
}

pub fn set_last_revenue_share_snapshot(env: &Env, ledger: u64) {
    env.storage()
        .instance()
        .set(&StorageKey::LastRevenueShareSnapshot, &ledger);
}

pub fn get_revenue_share_pool(env: &Env, token: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&StorageKey::RevenueSharePool(token.clone()))
        .unwrap_or(0)
}

pub fn add_revenue_share_pool(env: &Env, token: &Address, amount: i128) {
    let current: i128 = env
        .storage()
        .persistent()
        .get(&StorageKey::RevenueSharePool(token.clone()))
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&StorageKey::RevenueSharePool(token.clone()), &current.saturating_add(amount));
}

pub fn clear_revenue_share_pool(env: &Env, token: &Address) {
    env.storage()
        .persistent()
        .remove(&StorageKey::RevenueSharePool(token.clone()));
}
