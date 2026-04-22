use soroban_sdk::{contracterror, contracttype};

/// Populated when [`ContractError::InsufficientBalance`] is returned from
/// [`crate::TradeExecutorContract::execute_copy_trade`].
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsufficientBalanceDetail {
    pub required: i128,
    pub available: i128,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    NotInitialized = 1,
    PositionLimitReached = 2,
    InsufficientBalance = 3,
    InvalidAmount = 4,
    SlippageExceeded = 5,
    ReentrancyDetected = 6,
    Unauthorized = 7,
    TradeNotFound = 8,
}
