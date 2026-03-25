use soroban_sdk::{contracttype, String, Env};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PauseState {
    pub paused: bool,
    pub paused_at: u64,
    pub auto_unpause_at: Option<u64>,
    pub reason: String,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CircuitBreakerStats {
    pub attempts_window: u32,
    pub failures_window: u32,
    pub window_start: u64,
    pub volume_1h: i128,
    pub volume_24h_avg: i128,
    pub last_price: i128,
    pub last_price_time: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CircuitBreakerConfig {
    pub volume_spike_mult: u32,    // e.g. 10 for 10x
    pub max_failure_rate_bps: u32, // e.g. 5000 for 50%
    pub max_price_move_bps: u32,   // e.g. 3000 for 30%
    pub max_loss_1h: i128,         // e.g. 100,000 * 10^7
}

pub fn check_thresholds(
    env: &Env,
    stats: &CircuitBreakerStats,
    config: &CircuitBreakerConfig,
    current_price: i128,
) -> Option<String> {
    let now = env.ledger().timestamp();

    // 1. Mass Failures: >50% trade failure rate in a 10-minute window
    if stats.attempts_window >= 5 && now < stats.window_start + 600 {
        let failure_rate_bps = (stats.failures_window as u32 * 10000) / stats.attempts_window;
        if failure_rate_bps > config.max_failure_rate_bps {
            return Some(String::from_str(env, "High failure rate"));
        }
    }

    // 2. Volume Spike: Current hour volume > 10x the 24-hour average
    if stats.volume_24h_avg > 0 && stats.volume_1h > stats.volume_24h_avg * (config.volume_spike_mult as i128) {
        return Some(String::from_str(env, "Volume spike detected"));
    }

    // 3. Price Manipulation: Asset price moves >30% within 5 minutes
    if stats.last_price > 0 && now < stats.last_price_time + 300 {
        let price_diff = (current_price - stats.last_price).abs();
        let price_move_bps = (price_diff * 10000) / stats.last_price;
        if price_move_bps > config.max_price_move_bps as i128 {
            return Some(String::from_str(env, "Extreme price movement"));
        }
    }

    // 4. Loss Threshold
    if stats.volume_1h > config.max_loss_1h {
        return Some(String::from_str(env, "Loss threshold exceeded"));
    }

    None
}

impl Default for PauseState {
    fn default() -> Self {
        panic!("PauseState must be initialized explicitly");
    }
}

pub const CAT_TRADING: &str = "trading";
pub const CAT_SIGNALS: &str = "signals";
pub const CAT_STAKES: &str = "stakes";
pub const CAT_ALL: &str = "all";
