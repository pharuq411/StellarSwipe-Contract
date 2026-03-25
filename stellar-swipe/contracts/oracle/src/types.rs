 feature/emergency-pause-circuit-breaker
use soroban_sdk::{contracttype, Address, Vec, Env, Bytes, crypto::Ed25519Signature};
use stellar_swipe_common::{AssetPair};

 main
use crate::errors::OracleError;
use common::AssetPair;
use soroban_sdk::{contracttype, crypto::Ed25519Signature, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug)]
pub struct OracleReputation {
    pub total_submissions: u32,
    pub accurate_submissions: u32,
    pub avg_deviation: i128,
    pub reputation_score: u32,
    pub weight: u32,
    pub last_slash: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PriceSubmission {
    pub oracle: Address,
    pub price: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PriceData {
    pub asset_pair: AssetPair,
    pub price: i128,
    pub timestamp: u64,
    pub source: Address,
    pub confidence: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum StorageKey {
    Admin,
    PriceMap(AssetPair),
    OracleStats,
    Oracles,
    PriceSubmissions,
    ConsensusPrice,
    PauseStates,
    OracleWeight(Address),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ConsensusPriceData {
    pub price: i128,
    pub timestamp: u64,
    pub num_oracles: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ExternalPrice {
    pub asset_pair: AssetPair,
    pub price: i128,
    pub timestamp: u64,
    pub round_id: u64,
 feature/emergency-pause-circuit-breaker
    pub signature: Bytes, 

    pub signature: Vec<u8>,
 main
    pub oracle_address: Address,
}
