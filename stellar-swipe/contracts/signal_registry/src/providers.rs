use soroban_sdk::{contracttype, Address, Env, String, Vec};

use crate::types::ProviderPerformance;

pub const GOLD_TIER_STAKE: i128 = 1_000_000_000;
pub const MIN_CLOSED_SIGNALS: u32 = 20;
pub const MIN_SUCCESS_RATE_BPS: u32 = 6_000;

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationEligibility {
    pub eligible: bool,
    pub stake_ok: bool,
    pub history_ok: bool,
    pub success_rate_ok: bool,
    pub missing_criteria: Vec<String>,
}

pub fn check_verification_eligibility(
    env: &Env,
    provider: Address,
    stake: i128,
    stats: ProviderPerformance,
) -> VerificationEligibility {
    let stake_ok = stake >= GOLD_TIER_STAKE;
    let history_ok = stats.total_signals >= MIN_CLOSED_SIGNALS;
    let success_rate_ok = stats.success_rate >= MIN_SUCCESS_RATE_BPS;
    let eligible = stake_ok && history_ok && success_rate_ok;

    let mut missing_criteria = Vec::new(env);
    if !stake_ok {
        missing_criteria.push_back(String::from_str(env, "gold_tier_stake"));
    }
    if !history_ok {
        missing_criteria.push_back(String::from_str(env, "closed_signals"));
    }
    if !success_rate_ok {
        missing_criteria.push_back(String::from_str(env, "success_rate"));
    }

    crate::events::emit_verification_eligibility_checked(env, provider, eligible);

    VerificationEligibility {
        eligible,
        stake_ok,
        history_ok,
        success_rate_ok,
        missing_criteria,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn stats(total_signals: u32, success_rate: u32) -> ProviderPerformance {
        ProviderPerformance {
            total_signals,
            successful_signals: 0,
            failed_signals: 0,
            total_copies: 0,
            success_rate,
            avg_return: 0,
            total_volume: 0,
        }
    }

    #[test]
    fn fully_eligible_provider_passes() {
        let env = Env::default();
        let provider = Address::generate(&env);

        let eligibility = check_verification_eligibility(
            &env,
            provider,
            GOLD_TIER_STAKE,
            stats(MIN_CLOSED_SIGNALS, MIN_SUCCESS_RATE_BPS),
        );

        assert!(eligibility.eligible);
        assert!(eligibility.stake_ok);
        assert!(eligibility.history_ok);
        assert!(eligibility.success_rate_ok);
        assert_eq!(eligibility.missing_criteria.len(), 0);
    }

    #[test]
    fn partially_eligible_provider_reports_missing_criteria() {
        let env = Env::default();
        let provider = Address::generate(&env);

        let eligibility = check_verification_eligibility(
            &env,
            provider,
            GOLD_TIER_STAKE,
            stats(MIN_CLOSED_SIGNALS - 1, MIN_SUCCESS_RATE_BPS),
        );

        assert!(!eligibility.eligible);
        assert!(eligibility.stake_ok);
        assert!(!eligibility.history_ok);
        assert!(eligibility.success_rate_ok);
        assert_eq!(eligibility.missing_criteria.len(), 1);
    }

    #[test]
    fn not_eligible_provider_reports_all_missing_criteria() {
        let env = Env::default();
        let provider = Address::generate(&env);

        let eligibility = check_verification_eligibility(&env, provider, 0, stats(0, 0));

        assert!(!eligibility.eligible);
        assert!(!eligibility.stake_ok);
        assert!(!eligibility.history_ok);
        assert!(!eligibility.success_rate_ok);
        assert_eq!(eligibility.missing_criteria.len(), 3);
    }
}
